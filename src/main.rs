use service::service;

#[service]
trait MyService {
    async fn send_message();
}

fn main() {}
