use std::io;

#[service::service]
trait MyService {
    async fn send_message();

    async fn send_event(a: String);
}

#[derive(Clone)]
struct SomeServer;

impl MyService for SomeServer {
    fn send_message(self) {
        println!("called send_message");
    }

    fn send_event(self) {
        println!("Called send_event");
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    server::WsServer::listen("127.0.0.1:3000", SomeServer.serve()).await?;

    Ok(())
}
