use cri::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Server::start().await?;
    Ok(())
}
