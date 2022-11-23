use std::net::ToSocketAddrs;

use ws::{connect, Handler, Handshake, Message, Result, Sender};

struct Client {
    out: Sender,
}

impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        println!("Connection opened");
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("Message: {:?}", msg);
        Ok(())
    }
}

pub struct WsClient;

impl WsClient {
    pub fn connect<A>(addr: A) -> tokio::sync::mpsc::Sender<Vec<u8>>
    where
        A: ToSocketAddrs + 'static + std::borrow::Borrow<str> + Send,
    {
        let (msg_sender, mut msg_receiver) = tokio::sync::mpsc::channel(2);
        let (tx, rx) = std::sync::mpsc::channel();

        tokio::spawn(async move {
            if let Err(error) = connect(addr, |out| {
                tx.send(out.clone()).unwrap();

                Client { out }
            }) {
                println!("Failed to create WebSocket due to: {:?}", error);
            }
        });

        let ws_sender = rx.recv().unwrap();

        tokio::spawn(async move {
            while let Some(msg) = msg_receiver.recv().await {
                println!("Send Message: {:?}", msg);
                ws_sender.send(msg).unwrap();
            }
        });

        msg_sender
    }
}
