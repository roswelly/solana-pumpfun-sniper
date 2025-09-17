use crate::error::{Result, SniperError};
use crate::geyser::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tokio::time;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::Request;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct GrpcEndpoint {
    pub url: String,
    pub auth_token: String,
    pub priority: u8, // Lower number = higher priority
    pub weight: f64,   // Weight for load balancing
    pub enabled: bool,
}

#[derive(Debug)]
pub struct GrpcConnection {
    pub endpoint: GrpcEndpoint,
    pub client: GeyserClient<Channel>,
    pub last_health_check: Instant,
    pub is_healthy: bool,
    pub connection_id: u32,
}

pub struct GrpcManager {
    connections: Arc<RwLock<HashMap<u32, GrpcConnection>>>,
    endpoints: Vec<GrpcEndpoint>,
    tx_sender: broadcast::Sender<SubscribeResponse>,
    health_check_interval: Duration,
    failover_threshold: Duration,
}

impl GrpcManager {
    pub fn new(endpoints: Vec<GrpcEndpoint>) -> Self {
        let (tx_sender, _) = broadcast::channel(1000);
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            endpoints,
            tx_sender,
            health_check_interval: Duration::from_secs(30),
            failover_threshold: Duration::from_secs(60),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing gRPC connections...");
        
        for (index, endpoint) in self.endpoints.iter().enumerate() {
            if endpoint.enabled {
                match self.create_connection(endpoint.clone(), index as u32).await {
                    Ok(connection) => {
                        let mut connections = self.connections.write().await;
                        connections.insert(index as u32, connection);
                        info!("Connected to gRPC endpoint: {}", endpoint.url);
                    }
                    Err(e) => {
                        error!("Failed to connect to gRPC endpoint {}: {}", endpoint.url, e);
                    }
                }
            }
        }

        if self.connections.read().await.is_empty() {
            return Err(SniperError::Grpc(tonic::Status::unavailable(
                "No healthy gRPC connections available"
            )));
        }

        // Start health check task
        self.start_health_check_task().await;
        
        // Start subscription tasks
        self.start_subscription_tasks().await;

        Ok(())
    }

    async fn create_connection(&self, endpoint: GrpcEndpoint, connection_id: u32) -> Result<GrpcConnection> {
        let channel = Channel::from_shared(endpoint.url.clone())
            .map_err(|e| SniperError::Grpc(tonic::Status::from_error(e)))?
            .connect()
            .await
            .map_err(|e| SniperError::Grpc(tonic::Status::from_error(e)))?;

        let mut client = GeyserClient::new(channel);
        
        // Test connection with a simple request
        let test_request = SubscribeRequest {
            transactions: HashMap::new(),
            transactions_status: HashMap::new(),
            commitment: CommitmentLevel::Processed as i32,
        };

        // Note: We don't actually send the test request here as it would start a stream
        // Instead, we'll rely on health checks

        Ok(GrpcConnection {
            endpoint,
            client,
            last_health_check: Instant::now(),
            is_healthy: true,
            connection_id,
        })
    }

    async fn start_health_check_task(&self) {
        let connections = Arc::clone(&self.connections);
        let health_check_interval = self.health_check_interval;
        let failover_threshold = self.failover_threshold;

        tokio::spawn(async move {
            let mut interval = time::interval(health_check_interval);
            
            loop {
                interval.tick().await;
                
                let mut connections_guard = connections.write().await;
                let mut to_remove = Vec::new();
                
                for (id, connection) in connections_guard.iter_mut() {
                    // Simple health check - if we haven't received data recently, mark as unhealthy
                    if connection.last_health_check.elapsed() > failover_threshold {
                        connection.is_healthy = false;
                        warn!("gRPC connection {} marked as unhealthy", id);
                    }
                }
                
                // Remove unhealthy connections
                for id in to_remove {
                    connections_guard.remove(&id);
                }
            }
        });
    }

    async fn start_subscription_tasks(&self) {
        let connections = Arc::clone(&self.connections);
        let tx_sender = self.tx_sender.clone();

        tokio::spawn(async move {
            let mut connections_guard = connections.read().await;
            
            for (id, connection) in connections_guard.iter() {
                if connection.is_healthy {
                    let tx_sender_clone = tx_sender.clone();
                    let connection_id = connection.connection_id;
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection_stream(connection_id, tx_sender_clone).await {
                            error!("Connection {} stream error: {}", connection_id, e);
                        }
                    });
                }
            }
        });
    }

    async fn handle_connection_stream(
        connection_id: u32,
        tx_sender: broadcast::Sender<SubscribeResponse>,
    ) -> Result<()> {
        // This would be implemented to handle the actual stream
        // For now, it's a placeholder
        Ok(())
    }

    pub async fn subscribe(&self, request: SubscribeRequest) -> Result<()> {
        let connections = self.connections.read().await;
        
        // Find the best connection (highest priority, healthy)
        let best_connection = connections
            .values()
            .filter(|conn| conn.is_healthy)
            .min_by_key(|conn| conn.endpoint.priority);

        match best_connection {
            Some(connection) => {
                let mut client = connection.client.clone();
                
                tokio::spawn(async move {
                    match client.subscribe(Request::new(request)).await {
                        Ok(mut stream) => {
                            while let Some(response) = stream.message().await.unwrap_or(None) {
                                if let Err(e) = tx_sender.send(response) {
                                    error!("Failed to broadcast message: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Subscription error: {}", e);
                        }
                    }
                });
                
                Ok(())
            }
            None => Err(SniperError::Grpc(tonic::Status::unavailable(
                "No healthy connections available"
            ))),
        }
    }

    pub fn get_message_receiver(&self) -> broadcast::Receiver<SubscribeResponse> {
        self.tx_sender.subscribe()
    }

    pub async fn get_connection_stats(&self) -> Vec<ConnectionStats> {
        let connections = self.connections.read().await;
        
        connections
            .values()
            .map(|conn| ConnectionStats {
                connection_id: conn.connection_id,
                url: conn.endpoint.url.clone(),
                priority: conn.endpoint.priority,
                is_healthy: conn.is_healthy,
                last_health_check: conn.last_health_check,
                uptime: conn.last_health_check.elapsed(),
            })
            .collect()
    }

    pub async fn add_endpoint(&mut self, endpoint: GrpcEndpoint) -> Result<()> {
        let connection_id = self.endpoints.len() as u32;
        self.endpoints.push(endpoint.clone());
        
        match self.create_connection(endpoint, connection_id).await {
            Ok(connection) => {
                let mut connections = self.connections.write().await;
                connections.insert(connection_id, connection);
                info!("Added new gRPC endpoint: {}", endpoint.url);
                Ok(())
            }
            Err(e) => {
                error!("Failed to add gRPC endpoint {}: {}", endpoint.url, e);
                Err(e)
            }
        }
    }

    pub async fn remove_endpoint(&mut self, url: &str) -> Result<()> {
        if let Some(index) = self.endpoints.iter().position(|ep| ep.url == url) {
            self.endpoints.remove(index);
            
            let mut connections = self.connections.write().await;
            connections.remove(&(index as u32));
            
            info!("Removed gRPC endpoint: {}", url);
        }
        
        Ok(())
    }

    pub async fn rebalance_connections(&self) -> Result<()> {
        let connections = self.connections.read().await;
        let healthy_connections: Vec<_> = connections
            .values()
            .filter(|conn| conn.is_healthy)
            .collect();

        if healthy_connections.is_empty() {
            return Err(SniperError::Grpc(tonic::Status::unavailable(
                "No healthy connections for rebalancing"
            )));
        }

        // Implement load balancing logic here
        // For now, just log the current state
        info!("Rebalancing {} healthy connections", healthy_connections.len());
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct ConnectionStats {
    pub connection_id: u32,
    pub url: String,
    pub priority: u8,
    pub is_healthy: bool,
    pub last_health_check: Instant,
    pub uptime: Duration,
}

pub struct LoadBalancer {
    connections: Vec<ConnectionInfo>,
    current_index: usize,
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    pub id: u32,
    pub weight: f64,
    pub is_healthy: bool,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            current_index: 0,
        }
    }

    pub fn add_connection(&mut self, id: u32, weight: f64, is_healthy: bool) {
        self.connections.push(ConnectionInfo {
            id,
            weight,
            is_healthy,
        });
    }

    pub fn get_next_connection(&mut self) -> Option<u32> {
        let healthy_connections: Vec<_> = self.connections
            .iter()
            .filter(|conn| conn.is_healthy)
            .collect();

        if healthy_connections.is_empty() {
            return None;
        }

        // Simple round-robin for now
        let connection = &healthy_connections[self.current_index % healthy_connections.len()];
        self.current_index += 1;
        
        Some(connection.id)
    }

    pub fn update_connection_health(&mut self, id: u32, is_healthy: bool) {
        if let Some(conn) = self.connections.iter_mut().find(|c| c.id == id) {
            conn.is_healthy = is_healthy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_balancer() {
        let mut balancer = LoadBalancer::new();
        balancer.add_connection(1, 1.0, true);
        balancer.add_connection(2, 1.0, true);
        
        assert_eq!(balancer.get_next_connection(), Some(1));
        assert_eq!(balancer.get_next_connection(), Some(2));
    }

    #[test]
    fn test_grpc_endpoint() {
        let endpoint = GrpcEndpoint {
            url: "https://example.com".to_string(),
            auth_token: "token".to_string(),
            priority: 1,
            weight: 1.0,
            enabled: true,
        };
        
        assert!(endpoint.enabled);
        assert_eq!(endpoint.priority, 1);
    }
}
