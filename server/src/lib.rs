use std::io;
use std::net::ToSocketAddrs;
use ws::{listen, Handler, Message, Sender};

pub trait HandleIncoming {
    fn handle_request(self, req: Vec<u8>);
}

struct Server<S> {
    out: Sender,
    serve: S,
}

impl<S> Handler for Server<S>
where
    S: HandleIncoming + Clone,
{
    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        println!("New connection");

        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        match msg {
            Message::Binary(b) => {
                self.serve.clone().handle_request(b);
            }
            Message::Text(t) => {
                println!("text message: {}", t);
                println!("should get binary");
            }
        }

        Ok(())
    }
}

pub struct WsServer {}

impl WsServer {
    pub async fn listen<A, S>(addr: A, serve: S) -> io::Result<()>
    where
        A: ToSocketAddrs + std::fmt::Debug + Send + 'static,
        S: HandleIncoming + Clone + std::marker::Send + 'static,
    {
        tokio::spawn(async move {
            if let Err(error) = listen(addr, |out| Server {
                out,
                serve: serve.clone(),
            }) {
                eprintln!("Error: {:?}", error);
            };
        });

        Ok(())
    }
}
