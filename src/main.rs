#[service::service]
trait MyService {
    async fn send_message();

    async fn send_event(a: String);
}

struct SomeServer;

impl MyService for SomeServer {
    fn send_message(self) {
        println!("called send_message");
    }

    fn send_event(self) {
        println!("Called send_event");
    }
}

fn main() {
    let server = SomeServer;

    let server1 = server.serve();

    // server1.serve(MyServiceMethods::SendMessage);
    server1.handle_request(MyServiceMethods::SendEvent);
}
