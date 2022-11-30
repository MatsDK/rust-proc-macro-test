use server::WsServer;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    WsServer::listen("127.0.0.1:3000").await?;
    Ok(())
}
