use std::io;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};

#[service::service]
trait MyService {
    async fn send_event(conf: String);
    async fn send_message();
}

#[derive(Clone)]
struct MyServiceResolver;
impl MyService for MyServiceResolver {
    fn send_event(self, arg: String) {
        println!("send event called {arg}");
    }

    fn send_message(self) {
        println!("send message called");
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let client = MyServiceClient::new("ws://127.0.0.1:3000", MyServiceResolver.serve()).await?;

    let mut stdin = BufReader::new(stdin()).lines();

    while let Ok(line) = stdin.next_line().await {
        client.send_event(line.unwrap()).await?;
        // client.send_message().await?;
    }

    Ok(())
}
