use rust_ttd::web_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚂 Starting RusTTD Web Server...");
    web_server::run_server().await
}