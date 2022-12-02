use std::io;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Test {
    prop: String,
}

#[service::service]
trait MyService {
    async fn send_event(conf: String);
    async fn send_message(t: Test);
}

#[derive(Clone)]
struct MyServiceResolver;

#[service::server]
impl MyService for MyServiceResolver {
    fn send_event(self, arg: String) {
        println!("send event called {arg}");
    }

    async fn send_message(self, t: Test) {
        println!("send message called {:?}", t.prop);
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let client = MyServiceClient::new("ws://127.0.0.1:3000", MyServiceResolver.build()).await?;

    let mut stdin = BufReader::new(stdin()).lines();
    while let Ok(line) = stdin.next_line().await {
        // client.send_event(line.unwrap()).await?;
        client
            .send_message(Test {
                prop: line.unwrap(),
            })
            .await?;
    }

    Ok(())
}
