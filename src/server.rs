//! accepts new connections until two clients are connected
//! generated two 3types tuples and sends them to each clients
//! wait for decision both clients
//! computes winner and sends results

#![allow(dead_code, unused_imports, unused_variables)]
use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::{
    collections::{BTreeMap, HashMap},
    env,
    error::Error,
    io::Error as IoError,
    mem,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures::future::join_all;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type ClientsArc = Arc<Mutex<Clients>>;
type TreeType = BTreeMap<&'static str, Vec<&'static str>>;

const TYPES: [&str; 18] = [
    "bug", "dark", "dragon", "electric", "fairy", "fighting", "fire", "flying", "ghost", "grass",
    "ground", "ice", "normal", "poison", "psychic", "rock", "steel", "water",
];

enum Action {
    Ready,
    Selected(String),
    Error,
}

struct Client {
    addr: SocketAddr,
    tx: Tx,
    choices: Option<Vec<String>>,
    selected: Option<String>,
    ready: bool,
}

impl Client {
    fn new(addr: SocketAddr, tx: Tx) -> Client {
        Client {
            addr,
            tx,
            choices: None,
            selected: None,
            ready: false,
        }
    }

    fn set_choices(&mut self, rng: &mut ThreadRng) {
        self.choices = Some(
            TYPES
                .choose_multiple(rng, 3)
                .cloned()
                .map(|x| x.to_string())
                .collect(),
        );
    }

    fn win(&self) {
        let msg = "won".to_string();
        self.tx
            .unbounded_send(tungstenite::Message::Text(msg))
            .unwrap();
    }

    fn loose(&self) {
        let msg = "lost".to_string();
        self.tx
            .unbounded_send(tungstenite::Message::Text(msg))
            .unwrap();
    }

    fn tie(&self) {
        let msg = "tie".to_string();
        self.tx
            .unbounded_send(tungstenite::Message::Text(msg))
            .unwrap();
    }
}

struct Clients {
    p1: Option<Client>,
    p2: Option<Client>,
}

impl Clients {
    fn default() -> Clients {
        Clients { p1: None, p2: None }
    }

    fn add(&mut self, client: Client) {
        if self.p1.is_none() {
            self.p1 = Some(client)
        } else if self.p2.is_none() {
            self.p2 = Some(client)
        } else {
            panic!("clients is full.");
        }
    }

    fn get_other(&self, addr: SocketAddr) -> Option<Tx> {
        if self.p1.is_some() && self.p1.as_ref().unwrap().addr != addr {
            return Some(self.p1.as_ref().unwrap().tx.clone());
        }
        if self.p2.is_some() && self.p2.as_ref().unwrap().addr != addr {
            return Some(self.p2.as_ref().unwrap().tx.clone());
        }

        None
    }

    fn set_ready(&mut self, addr: SocketAddr) {
        if let Some(ref mut p1) = self.p1 {
            p1.ready = p1.ready || p1.addr == addr;
        }

        if let Some(ref mut p2) = self.p2 {
            p2.ready = p2.ready || p2.addr == addr;
        }
    }

    fn both_ready(&self) -> bool {
        if let Some(ref p1) = self.p1 {
            if let Some(ref p2) = self.p2 {
                if p1.ready && p2.ready {
                    return true;
                }
            }
        }
        false
    }

    fn send_choices(&mut self) {
        let mut rng = rand::thread_rng();

        let p1_choices = match self.p1 {
            Some(ref mut p) => {
                p.set_choices(&mut rng);
                p.choices.as_ref().unwrap().join(",")
            }
            _ => panic!("Unset client 'p1' cannot get choices !"),
        };

        let p2_choices = match self.p2 {
            Some(ref mut p) => {
                p.set_choices(&mut rng);
                p.choices.as_ref().unwrap().join(",")
            }
            _ => panic!("Unset client 'p2' cannot get choices !"),
        };

        if let Some(ref p) = self.p1 {
            let msg = format!("yours:{};theirs:{}", p1_choices, p2_choices);
            p.tx.unbounded_send(tungstenite::Message::Text(msg))
                .unwrap();
        }

        if let Some(ref p) = self.p2 {
            let msg = format!("yours:{};theirs:{}", p2_choices, p1_choices);
            p.tx.unbounded_send(tungstenite::Message::Text(msg))
                .unwrap();
        }
    }

    fn set_selected(&mut self, addr: SocketAddr, type_: String) {
        if let Some(ref mut p) = self.p1 {
            if p.addr == addr {
                p.selected = Some(type_.clone());
            }
        }

        if let Some(ref mut p) = self.p2 {
            if p.addr == addr {
                p.selected = Some(type_);
            }
        }
    }

    fn both_selected(&self) -> bool {
        if let Some(ref p1) = self.p1 {
            if let Some(ref p2) = self.p2 {
                if p1.selected.is_some() && p2.selected.is_some() {
                    return true;
                }
            }
        }
        false
    }

    fn get_selected(&self) -> Option<(String, String)> {
        let p1_selected: String;
        let p2_selected: String;
        if let Some(ref p1) = self.p1 {
            if let Some(ref p1s) = p1.selected {
                p1_selected = p1s.clone();
            } else {
                return None;
            }
        } else {
            return None;
        }

        if let Some(ref p2) = self.p2 {
            if let Some(ref p2s) = p2.selected {
                p2_selected = p2s.clone();
            } else {
                return None;
            }
        } else {
            return None;
        }

        Some((p1_selected, p2_selected))
    }

    fn send_outcome(&self) {
        let mut p1_score = 1;
        let mut p2_score = 1;

        let strengths = make_strengths_graph();
        let weaknesses = make_weaknesses_graph();

        if let Some((p1_selected, p2_selected)) = self.get_selected() {
            if strengths[&*p1_selected].contains(&&*p2_selected) {
                p1_score *= 2;
            }
            if weaknesses[&*p1_selected].contains(&&*p2_selected) {
                p1_score /= 2;
            }

            if strengths[&*p2_selected].contains(&&*p1_selected) {
                p2_score *= 2;
            }
            if weaknesses[&*p2_selected].contains(&&*p1_selected) {
                p2_score /= 2;
            }
        }

        let p1 = self.p1.as_ref().unwrap();
        let p2 = self.p2.as_ref().unwrap();

        println!("p1: {} vs p2: {}", p1_score, p2_score);

        if p1_score == p2_score {
            p1.tie();
            p2.tie();
        } else if p1_score > p2_score {
            p1.win();
            p2.loose();
        } else {
            p1.loose();
            p2.win();
        }
    }

    fn send_msg(&self, addr: SocketAddr, msg: String) {
        if let Some(p1) = &self.p1 {
            if p1.addr == addr {
                p1.tx
                    .unbounded_send(tungstenite::Message::Text(msg.clone()))
                    .unwrap();
            }
        }

        if let Some(p2) = &self.p2 {
            if p2.addr == addr {
                p2.tx
                    .unbounded_send(tungstenite::Message::Text(msg))
                    .unwrap();
            }
        }
    }
}

fn make_strengths_graph() -> TreeType {
    let mut w = BTreeMap::<&str, Vec<&'static str>>::new();
    w.insert("bug", vec!["dark", "grass", "psychic"]);
    w.insert("dark", vec!["psychic", "ghost"]);
    w.insert("dragon", vec!["dragon"]);
    w.insert("electric", vec!["water", "flying"]);
    w.insert("fairy", vec!["dragon", "fighting", "dark"]);
    w.insert("fighting", vec!["rock", "normal", "dark", "steel", "ice"]);
    w.insert("fire", vec!["ice", "grass", "bug", "steel"]);
    w.insert("flying", vec!["grass", "fighting", "bug"]);
    w.insert("ghost", vec!["ghost", "psychic"]);
    w.insert("grass", vec!["rock", "ground", "water"]);
    w.insert("ground", vec!["steel", "rock", "fire", "poison"]);
    w.insert("ice", vec!["flying", "grass", "ground", "dragon"]);
    w.insert("poison", vec!["grass", "fairy"]);
    w.insert("psychic", vec!["fighting", "poison"]);
    w.insert("rock", vec!["bug", "flying", "fire"]);
    w.insert("steel", vec!["fairy", "ice", "rock"]);
    w.insert("water", vec!["ground", "fire", "rock"]);
    w.insert("normal", vec![]);
    w
}

fn make_weaknesses_graph() -> TreeType {
    let mut w = BTreeMap::<&str, Vec<&'static str>>::new();
    w.insert(
        "bug",
        vec![
            "fire", "fighting", "flying", "poison", "ghost", "steel", "fairy",
        ],
    );
    w.insert("dark", vec!["fighting", "dark", "fairy"]);
    w.insert("dragon", vec!["steel", "fairy"]);
    w.insert("electric", vec!["electric", "grass", "dragon", "ground"]);
    w.insert("fairy", vec!["fire", "poison", "steel"]);
    w.insert(
        "fighting",
        vec!["ghost", "flying", "psychic", "bug", "fairy"],
    );
    w.insert("fire", vec!["fire", "water", "rock", "dragon"]);
    w.insert("flying", vec!["electric", "rock", "steel"]);
    w.insert("ghost", vec!["dark", "normal"]);
    w.insert(
        "grass",
        vec![
            "fire", "grass", "poison", "flying", "bug", "poison", "steel",
        ],
    );
    w.insert("ground", vec!["grass", "bug", "flying"]);
    w.insert("ice", vec!["ice", "fire", "water", "steel"]);
    w.insert("poison", vec!["steel", "poison", "ground", "rock", "ghost"]);
    w.insert("psychic", vec!["dark", "psychic", "steel"]);
    w.insert("rock", vec!["fighting", "ground", "steel"]);
    w.insert("steel", vec!["fire", "water", "electric", "steel"]);
    w.insert("water", vec!["water", "grass", "dragon"]);
    w.insert("normal", vec!["ghost", "rock", "steel"]);
    w
}

fn parse(msg: tungstenite::Message) -> Action {
    let text = match msg.to_text() {
        Ok(text) => text,
        Err(_) => return Action::Error,
    };
    let full: Vec<&str> = text.splitn(2, ':').collect();

    if full.len() < 2 {
        return Action::Error;
    }

    let (action, parameters) = (full[0], full[1]);
    match action {
        "ready" => Action::Ready,
        "selected" => Action::Selected(parameters.to_string()),
        _ => Action::Error,
    }
}

async fn handle_connection(clients: ClientsArc, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let (tx, rx) = unbounded();
    clients.lock().unwrap().add(Client::new(addr, tx));

    let (outgoing, incoming) = ws_stream.split();

    let handle_incoming = incoming.try_for_each(|msg| {
        println!("Received a message from {}: {}", addr, msg);

        match parse(msg) {
            Action::Ready => {
                let mut c = clients.lock().unwrap();
                c.set_ready(addr);
                println!("{} is ready", addr);
                if c.both_ready() {
                    println!("both ready, sending choices.");
                    c.send_choices();
                }
            }
            Action::Selected(type_) => {
                let mut c = clients.lock().unwrap();
                c.set_selected(addr, type_.clone());
                println!("{} selected {}", addr, type_);
                if c.both_selected() {
                    println!("both selected, computing outcome.");
                    c.send_outcome();
                }
            }
            Action::Error => {
                println!("dafuk?");
                clients.lock().unwrap().send_msg(addr, "dafuk?".to_string());
            }
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(handle_incoming, receive_from_others);
    future::select(handle_incoming, receive_from_others).await;
    println!("{} disconnected", &addr);
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let clients = ClientsArc::new(Mutex::new(Clients::default()));

    let listener = (TcpListener::bind(&addr).await).expect("Failed to bind");
    let mut handles = vec![];

    while let Ok((stream, addr)) = listener.accept().await {
        handles.push(tokio::spawn(handle_connection(
            clients.clone(),
            stream,
            addr,
        )));

        if handles.len() >= 2 {
            break;
        }
    }

    join_all(handles).await;

    Ok(())
}
