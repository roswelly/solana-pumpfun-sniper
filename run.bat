@echo off
echo Building Solana PumpFun Sniper Bot...
cargo build --release

if %ERRORLEVEL% neq 0 (
    echo Build failed!
    pause
    exit /b 1
)

echo Build successful! Starting bot...
cargo run --release
