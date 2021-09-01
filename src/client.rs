mod libclient;

use crate::libclient::{
    drawing::{draw_choices, draw_outcome, retry},
    state::{GameState, Outcome},
    textures::TextureStore,
    utils::parse_choices,
};
use futures_channel::{
    mpsc,
    mpsc::{UnboundedReceiver, UnboundedSender},
};
use futures_util::{future, pin_mut, StreamExt};
use raylib::prelude::*;
use std::{
    cmp::{max, min},
    env,
};
use tokio::spawn;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

type ReadRx = UnboundedReceiver<Result<Message, TungsteniteError>>;
type WriteTx = UnboundedSender<Message>;

fn get_message(read_rx: &mut ReadRx) -> Option<String> {
    let message: String;
    match read_rx.try_next() {
        Ok(Some(msg)) => {
            println!("{:?}", msg);
            message = msg.unwrap().to_string();
            Some(message)
        }
        Ok(None) | Err(_) => None,
    }
}

fn handle_input(
    draw_handle: &mut RaylibDrawHandle,
    mine: &[String],
    hoover_index: usize,
) -> (Option<String>, Option<usize>) {
    if draw_handle.is_key_pressed(KeyboardKey::KEY_LEFT) {
        (None, Some(max(1, hoover_index) - 1))
    } else if draw_handle.is_key_pressed(KeyboardKey::KEY_RIGHT) {
        (None, Some(min(2, hoover_index + 1)))
    } else if draw_handle.is_key_pressed(KeyboardKey::KEY_ENTER) {
        (Some(mine[hoover_index].clone()), None)
    } else {
        (None, Some(hoover_index))
    }
}

fn parse_outcome(message: String) -> (Outcome, String, String) {
    let parsed: Vec<String> = message.splitn(3, ';').map(String::from).collect();

    let (status, yours, theirs) = (parsed[0].clone(), parsed[1].clone(), parsed[2].clone());

    let outcome = match &*status {
        "won" => Outcome::Won,
        "lost" => Outcome::Lost,
        "tie" => Outcome::Tie,
        _ => panic!("wtf?"),
    };
    (outcome, yours, theirs)
}

async fn main_loop(mut read_rx: ReadRx, write_tx: WriteTx) {
    let mut gamestate = GameState::WaitingForChoices;
    let mut scores = (0, 0);

    write_tx
        .unbounded_send(Message::Text("ready:_".to_string()))
        .unwrap();
    println!("ready");

    set_trace_log(TraceLogType::LOG_FATAL);
    let (mut handle, thread) = raylib::init().size(640, 480).title("Hello, World").build();
    handle.set_target_fps(60);
    let ts = TextureStore::new(&mut handle, &thread);

    while !handle.window_should_close() {
        let mut draw_handle = handle.begin_drawing(&thread);
        draw_handle.clear_background(Color::WHITE);

        draw_handle.draw_text(
            &*format!("{}/{}", scores.0, scores.1),
            615,
            10,
            10,
            Color::BLACK,
        );

        match gamestate {
            GameState::WaitingForChoices => {
                draw_handle.draw_text("Waiting for Opponent ...", 10, 10, 10, Color::BLACK);
                if let Some(choices) = get_message(&mut read_rx) {
                    let (mine, theirs) = parse_choices(choices);
                    gamestate = GameState::GotChoices(mine, theirs, 1);
                }
            }
            GameState::GotChoices(ref mine, ref theirs, hoover_index) => {
                draw_choices(&mut draw_handle, &ts, mine, theirs, hoover_index);

                match handle_input(&mut draw_handle, mine, hoover_index) {
                    (Some(selected), None) => {
                        let msg = format!("selected:{}", selected);
                        write_tx
                            .unbounded_send(Message::Text(msg.to_string()))
                            .unwrap();
                        gamestate = GameState::WaitingForOtherSelected;
                    }
                    (None, Some(hoover_index)) => {
                        gamestate =
                            GameState::GotChoices(mine.to_vec(), theirs.to_vec(), hoover_index);
                    }
                    _ => panic!("invalid state!"),
                }
            }
            GameState::WaitingForOtherSelected => {
                draw_handle.draw_text("Waiting for Opponent ...", 10, 10, 10, Color::BLACK);
                if let Some(message) = get_message(&mut read_rx) {
                    let (outcome, yours, theirs) = parse_outcome(message);
                    gamestate = GameState::GotOutcome(outcome, yours, theirs);
                }
            }
            GameState::GotOutcome(ref outcome, ref yours, ref theirs) => {
                draw_outcome(
                    &mut draw_handle,
                    outcome,
                    yours.to_string(),
                    theirs.to_string(),
                );
                if retry(&mut draw_handle) {
                    match outcome {
                        Outcome::Won => scores.0 += 1,
                        Outcome::Lost => scores.1 += 1,
                        _ => (),
                    }

                    gamestate = GameState::WaitingForChoices;
                    write_tx
                        .unbounded_send(Message::Text("ready:_".to_string()))
                        .unwrap();
                }
            }
        }
    }

    println!("done");
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "ws://127.0.0.1:8080/".to_string());
    let url = url::Url::parse(&addr).unwrap();

    let (ws, _) = connect_async(url).await.expect("Failed to connect");
    let (write, read) = ws.split();

    let (write_tx, write_rx) = mpsc::unbounded();
    let (read_tx, read_rx) = mpsc::unbounded();
    let read_handle = read.map(Ok).forward(read_tx);
    let write_handle = write_rx.map(Ok).forward(write);

    spawn(main_loop(read_rx, write_tx));

    pin_mut!(read_handle, write_handle);
    future::select(read_handle, write_handle).await;

    Ok(())
}
