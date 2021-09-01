use crate::libserver::utils::TYPES;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use futures_channel::mpsc::{UnboundedSender};
use tungstenite::protocol::Message;
use std::net::SocketAddr;

pub type Tx = UnboundedSender<Message>;

pub struct Client {
    pub addr: SocketAddr,
    pub tx: Tx,
    pub choices: Option<Vec<String>>,
    pub selected: Option<String>,
    pub ready: bool,
}

impl Client {
    pub fn new(addr: SocketAddr, tx: Tx) -> Client {
        Client {
            addr,
            tx,
            choices: None,
            selected: None,
            ready: false,
        }
    }

    pub fn set_choices(&mut self, rng: &mut ThreadRng) {
        self.choices = Some(
            TYPES
                .choose_multiple(rng, 3)
                .cloned()
                .map(|x| x.to_string())
                .collect(),
        );
    }

    pub fn send_outcome(&self, status: &'static str, yours: String, theirs: String) {
        let msg = format!("{};{};{}", status, yours, theirs);
        self.tx
            .unbounded_send(tungstenite::Message::Text(msg))
            .unwrap();
    }
}