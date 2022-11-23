use std::{io, thread, time::Duration};

use server::WsServer;

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
        println!("Called send_event21");
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    WsServer::listen("127.0.0.1:3000", SomeServer.serve()).await?;
    println!("server started");

    thread::sleep(Duration::from_millis(1000));

    println!("create client");
    let client = SomeServer.build_client();

    thread::sleep(Duration::from_millis(1000));

    println!("send messages");
    client.send_event().await?;
    client.send_message().await?;

    Ok(())
}
