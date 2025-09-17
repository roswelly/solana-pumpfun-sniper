fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create proto directory if it doesn't exist
    std::fs::create_dir_all("proto")?;
    
    // Check if proto files exist before trying to compile them
    if std::path::Path::new("proto/geyser.proto").exists() {
        tonic_build::configure()
            .build_server(false)
            .compile(&["proto/geyser.proto"], &["proto"])?;
        println!("cargo:rerun-if-changed=proto/geyser.proto");
    } else {
        // For now, skip proto compilation if file doesn't exist
        // The project will need to provide the actual Solana Geyser proto file
        println!("cargo:warning=No proto/geyser.proto file found. Please add the Solana Geyser proto definition.");
        println!("cargo:warning=You can download it from: https://github.com/solana-labs/solana/blob/master/geyser-plugin-interface/proto/geyser.proto");
    }
    
    Ok(())
}
