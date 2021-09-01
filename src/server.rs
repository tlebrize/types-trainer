mod lib;
use crate::lib::server::{compute_scores, TreeType};

use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::{
    collections::BTreeMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};

use futures::future::join_all;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};

use tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type ClientsArc = Arc<Mutex<Clients>>;

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
    timer: Option<DateTime<Utc>>,
}

impl Client {
    fn new(addr: SocketAddr, tx: Tx) -> Client {
        Client {
            addr,
            tx,
            choices: None,
            selected: None,
            timer: None,
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

    fn send_outcome(&self, status: &'static str, yours: String, theirs: String) {
        let msg = format!("{};{};{}", status, yours, theirs);
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

    fn send_outcomes(&self) {
        if let Some((p1_selected, p2_selected)) = self.get_selected() {
            let p1 = self.p1.as_ref().unwrap();
            let p2 = self.p2.as_ref().unwrap();

            let strengths = make_strengths_graph();
            let weaknesses = make_weaknesses_graph();

            let (p1_score, p2_score) = compute_scores(
                p1_selected.clone(),
                p2_selected.clone(),
                strengths,
                weaknesses,
            );

            println!("p1: {} vs p2: {}", p1_score, p2_score);

            if p1_score == p2_score {
                p1.send_outcome("tie", p1_selected.clone(), p2_selected.clone());
                p2.send_outcome("tie", p1_selected, p2_selected);
            } else if p1_score > p2_score {
                p1.send_outcome("won", p1_selected.clone(), p2_selected.clone());
                p2.send_outcome("lost", p2_selected, p1_selected);
            } else {
                p1.send_outcome("lost", p1_selected.clone(), p2_selected.clone());
                p2.send_outcome("won", p2_selected, p1_selected);
            }
        } else {
            panic!("Cannot find outcome !");
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

    fn reset(&mut self) {
        if let Some(ref mut p1) = self.p1 {
            p1.choices = None;
            p1.selected = None;
            p1.ready = false;
        }
        if let Some(ref mut p2) = self.p2 {
            p2.choices = None;
            p2.selected = None;
            p2.ready = false;
        }
    }

    fn set_timers(&mut self) {
        if let Some(ref mut p1) = self.p1 {
            p1.timer = Some(Utc::now());
        }

        if let Some(ref mut p2) = self.p2 {
            p2.timer = Some(Utc::now());
        }
    }

    fn stop_timer(&mut self, addr: SocketAddr) -> bool {
        if let Some(ref mut p1) = self.p1 {
            if p1.addr == addr && p1.timer.is_some() {
                let time_diff = p1
                    .timer
                    .unwrap()
                    .signed_duration_since(Utc::now())
                    .num_seconds();
                println!("{}:{}", addr, time_diff);
                p1.timer = None;
                return time_diff < -5;
            }
        }

        if let Some(ref mut p2) = self.p2 {
            if p2.addr == addr && p2.timer.is_some() {
                let time_diff = p2
                    .timer
                    .unwrap()
                    .signed_duration_since(Utc::now())
                    .num_seconds();
                println!("{}:{}", addr, time_diff);
                p2.timer = None;
                return time_diff > 15;
            }
        }

        panic!("invalid addr: {}. Or timers were not set.", addr);
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
                    c.set_timers();
                }
            }
            Action::Selected(type_) => {
                let mut c = clients.lock().unwrap();
                c.set_selected(addr, type_.clone());
                if c.stop_timer(addr) {
                    println!("{} Timed out !", addr);
                    c.send_msg(addr, "timedout;_;_".to_string());
                    c.reset();
                }
                println!("{} selected {}", addr, type_);
                if c.both_selected() {
                    println!("both selected, computing outcome.");
                    c.send_outcomes();
                    c.reset();
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
