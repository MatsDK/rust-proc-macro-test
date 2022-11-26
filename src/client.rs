use std::io;

#[service::service]
trait MyService {
    async fn send_event();
}

#[derive(Clone)]
struct MyServiceResolver;
impl MyService for MyServiceResolver {
    fn send_event(self) {
        println!("send event");
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let client = MyServiceClient::new(MyServiceResolver.serve()).await;

    client.send_event().await?;

    Ok(())
}
