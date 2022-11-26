use tokio::sync::mpsc;

pub struct Channel {
    pub tx: mpsc::Sender<Vec<u8>>,
}

impl Channel {
    pub fn new() -> (Self, mpsc::Receiver<Vec<u8>>) {
        let (tx, rx) = mpsc::channel(2);

        (Channel { tx }, rx)
    }
}

pub struct WsClient {}

impl WsClient {
    pub async fn new(mut event_channel_receiver: mpsc::Receiver<Vec<u8>>) -> Self {
        println!("build client");

        tokio::spawn(async move {
            while let Some(ev) = event_channel_receiver.recv().await {
                println!("{:?}", ev);
            }
        });

        Self {}
    }
}
