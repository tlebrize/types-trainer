mod libserver;
use crate::libserver::{
    client::Client,
    clients::Clients,
    utils::{compute_scores, make_strengths_graph, make_weaknesses_graph},
};

use std::{
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures::future::join_all;
use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};

type ClientsArc = Arc<Mutex<Clients>>;

enum Action {
    Ready,
    Selected(String),
    Error,
}

fn parse_action(msg: tungstenite::Message) -> Action {
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

        match parse_action(msg) {
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
