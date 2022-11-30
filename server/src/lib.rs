use std::io;
use std::net::ToSocketAddrs;
use ws::{listen, Handler, Message, Sender};

struct Server {
    out: Sender,
}

impl Handler for Server {
    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        println!("New connection");

        Ok(())
    }

    fn on_close(&mut self, _code: ws::CloseCode, reason: &str) {
        println!("Connection closed {reason}");
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        println!("got message, {:?}", msg);
        self.out.broadcast(msg)?;

        Ok(())
    }
}

pub struct WsServer;

impl WsServer {
    pub async fn listen<A>(addr: A) -> io::Result<()>
    where
        A: ToSocketAddrs + Send + 'static + std::fmt::Debug,
    {
        listen(addr, |out| Server { out }).unwrap();

        Ok(())
    }
}
