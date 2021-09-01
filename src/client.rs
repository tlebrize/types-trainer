mod libclient;

use crate::libclient::client::parse_choices;
use futures_channel::{
    mpsc,
    mpsc::{UnboundedReceiver, UnboundedSender},
};
use futures_util::{future, pin_mut, StreamExt};
use raylib::prelude::*;
use std::{
    cmp::{max, min},
    collections::BTreeMap,
    env,
};
use tokio::spawn;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

type ReadRx = UnboundedReceiver<Result<Message, TungsteniteError>>;
type WriteTx = UnboundedSender<Message>;

enum Outcome {
    Won,
    Lost,
    Tie,
}

enum GameState {
    WaitingForChoices,
    GotChoices(Vec<String>, Vec<String>, usize),
    WaitingForOtherSelected,
    GotOutcome(Outcome, String, String),
}

fn tex_rec() -> Rectangle {
    Rectangle {
        x: 0.0,
        y: 0.0,
        width: 70.0,
        height: 70.0,
    }
}

fn get_type_texture(
    type_: &str,
    handle: &mut raylib::RaylibHandle,
    thread: &RaylibThread,
) -> Result<Texture2D, String> {
    let type_filename = format!("media/{}.png", type_);
    let type_image = Image::load_image(&type_filename)?;
    handle.load_texture_from_image(thread, &type_image)
}

struct TextureStore {
    textures: BTreeMap<&'static str, Texture2D>,
}

impl TextureStore {
    fn new(handle: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let mut t = BTreeMap::new();

        t.insert("bug", get_type_texture("bug", handle, thread).unwrap());
        t.insert("dark", get_type_texture("dark", handle, thread).unwrap());
        t.insert(
            "dragon",
            get_type_texture("dragon", handle, thread).unwrap(),
        );
        t.insert(
            "electric",
            get_type_texture("electric", handle, thread).unwrap(),
        );
        t.insert("fairy", get_type_texture("fairy", handle, thread).unwrap());
        t.insert(
            "fighting",
            get_type_texture("fighting", handle, thread).unwrap(),
        );
        t.insert("fire", get_type_texture("fire", handle, thread).unwrap());
        t.insert(
            "flying",
            get_type_texture("flying", handle, thread).unwrap(),
        );
        t.insert("ghost", get_type_texture("ghost", handle, thread).unwrap());
        t.insert("grass", get_type_texture("grass", handle, thread).unwrap());
        t.insert(
            "ground",
            get_type_texture("ground", handle, thread).unwrap(),
        );
        t.insert("ice", get_type_texture("ice", handle, thread).unwrap());
        t.insert(
            "poison",
            get_type_texture("poison", handle, thread).unwrap(),
        );
        t.insert(
            "psychic",
            get_type_texture("psychic", handle, thread).unwrap(),
        );
        t.insert("rock", get_type_texture("rock", handle, thread).unwrap());
        t.insert("steel", get_type_texture("steel", handle, thread).unwrap());
        t.insert("water", get_type_texture("water", handle, thread).unwrap());
        t.insert(
            "normal",
            get_type_texture("normal", handle, thread).unwrap(),
        );

        TextureStore { textures: t }
    }
}

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

fn draw_choices(
    draw_handle: &mut RaylibDrawHandle,
    ts: &TextureStore,
    mine: &[String],
    theirs: &[String],
    i: usize,
) {
    draw_handle.draw_texture_rec(
        &ts.textures[&*mine[0]],
        tex_rec(),
        Vector2 { x: 50.0, y: 280.0 },
        if i == 0 { Color::GRAY } else { Color::WHITE },
    );
    draw_handle.draw_texture_rec(
        &ts.textures[&*mine[1]],
        tex_rec(),
        Vector2 { x: 285.0, y: 280.0 },
        if i == 1 { Color::GRAY } else { Color::WHITE },
    );
    draw_handle.draw_texture_rec(
        &ts.textures[&*mine[2]],
        tex_rec(),
        Vector2 { x: 520.0, y: 280.0 },
        if i == 2 { Color::GRAY } else { Color::WHITE },
    );

    draw_handle.draw_texture_rec(
        &ts.textures[&*theirs[0]],
        tex_rec(),
        Vector2 { x: 50.0, y: 50.0 },
        Color::WHITE,
    );
    draw_handle.draw_texture_rec(
        &ts.textures[&*theirs[1]],
        tex_rec(),
        Vector2 { x: 285.0, y: 50.0 },
        Color::WHITE,
    );
    draw_handle.draw_texture_rec(
        &ts.textures[&*theirs[2]],
        tex_rec(),
        Vector2 { x: 520.0, y: 50.0 },
        Color::WHITE,
    );
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

fn retry(draw_handle: &mut RaylibDrawHandle) -> bool {
    draw_handle.draw_text("Press enter to play again.", 10, 10, 10, Color::BLACK);
    draw_handle.is_key_pressed(KeyboardKey::KEY_ENTER)
}

fn draw_outcome(
    draw_handle: &mut RaylibDrawHandle,
    outcome: &Outcome,
    yours: String,
    theirs: String,
) {
    let (yours, theirs) = (&*yours, &*theirs);
    match outcome {
        Outcome::Won => {
            draw_handle.draw_text("You won !", 320, 240, 24, Color::BLACK);
            draw_handle.draw_text(
                &*format!("{} beats {}", yours, theirs),
                200,
                280,
                20,
                Color::BLACK,
            );
        }
        Outcome::Lost => {
            draw_handle.draw_text("You lost :/", 320, 240, 24, Color::BLACK);
            draw_handle.draw_text(
                &*format!("{} beats {}", theirs, yours),
                200,
                280,
                20,
                Color::BLACK,
            );
        }
        Outcome::Tie => {
            draw_handle.draw_text("Its a tie ...", 320, 240, 24, Color::BLACK);
            draw_handle.draw_text(
                &*format!("{} == {}", theirs, yours),
                200,
                280,
                20,
                Color::BLACK,
            );
        }
    };
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
