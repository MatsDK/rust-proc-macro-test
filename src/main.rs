#[service::service]
trait MyService {
    async fn send_message();

    // async fn send_event(a: String);
}

fn main() {}
