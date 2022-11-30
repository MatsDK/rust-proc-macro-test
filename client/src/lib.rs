use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, io};
use tokio::sync::mpsc;
use ws::{connect, CloseCode, Handler, Handshake, Message, Sender};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    sender: String,
    msg: Vec<u8>,
}

pub trait HandleIncoming {
    fn handle_incoming_event(self, req: Vec<u8>);
}

pub struct Channel {
    pub tx: mpsc::Sender<Vec<u8>>,
}

impl Channel {
    pub fn new() -> (Self, mpsc::Receiver<Vec<u8>>) {
        let (tx, rx) = mpsc::channel(2);

        (Channel { tx }, rx)
    }
}

struct Client<R> {
    out: Sender,
    resolvers: R,
    id: String,
}

impl<R> Handler for Client<R>
where
    R: HandleIncoming + Clone,
{
    fn on_open(&mut self, _shake: Handshake) -> ws::Result<()> {
        println!("connection opened");
        Ok(())
    }

    fn on_close(&mut self, _code: CloseCode, reason: &str) {
        println!("connection closed {}", reason);
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        match msg {
            Message::Binary(b) => match serde_json::from_slice::<Request>(&b) {
                Ok(r) => {
                    if r.sender != self.id {
                        self.resolvers.clone().handle_incoming_event(r.msg);
                    }
                }
                Err(e) => {
                    println!("Error parsing request {}", e);
                }
            },
            Message::Text(t) => {
                println!("text message: {}", t);
                println!("should get binary");
            }
        }

        Ok(())
    }
}

pub struct WsClient;

impl WsClient {
    pub async fn connect<A, R>(addr: A, resolvers: R) -> io::Result<Channel>
    where
        A: Borrow<str> + Send + 'static,
        R: HandleIncoming + Clone + Send + 'static,
    {
        let id = generate_random_string(32);

        let (channel, mut event_rx) = Channel::new();
        let (tx, rx) = std::sync::mpsc::channel();

        let client_id = id.clone();
        tokio::spawn(async move {
            if let Err(e) = connect(addr, |out| {
                tx.send(out.clone()).expect("failed to send 'out'");
                Client {
                    out,
                    resolvers: resolvers.clone(),
                    id: client_id.clone(),
                }
            }) {
                eprintln!("Error: {}", e);
            }
        });

        let ws_sender = rx.recv().unwrap();

        tokio::spawn(async move {
            while let Some(ev) = event_rx.recv().await {
                let req = Request {
                    sender: id.clone(),
                    msg: ev,
                };
                let req = serde_json::to_vec(&req).unwrap();

                ws_sender.send(req).unwrap();
            }
        });

        Ok(channel)
    }
}

fn generate_random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
