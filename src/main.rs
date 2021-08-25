use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use raylib::prelude::*;
use std::collections::BTreeMap;

type WeaknessesTreeType = BTreeMap<&'static str, Vec<&'static str>>;

const TYPES: [&str; 18] = [
    "bug", "dark", "dragon", "electric", "fairy", "fighting", "fire", "flying", "ghost", "grass",
    "ground", "ice", "normal", "poison", "psychic", "rock", "steel", "water",
];

fn make_graph() -> WeaknessesTreeType {
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

fn get_type_texture(
    type_: &str,
    handle: &mut raylib::RaylibHandle,
    thread: &RaylibThread,
) -> Result<Texture2D, String> {
    let type_filename = format!("media/{}.png", type_);
    let type_image = Image::load_image(&type_filename)?;
    handle.load_texture_from_image(thread, &type_image)
}

fn tex_rec() -> Rectangle {
    Rectangle {
        x: 0.0,
        y: 0.0,
        width: 70.0,
        height: 70.0,
    }
}

fn handle_input(weaknesses: &WeaknessesTreeType, attacker: &str, target: &str) -> bool {
    weaknesses[attacker].contains(&target)
}

fn main_loop(
    handle: &mut RaylibHandle,
    thread: &RaylibThread,
    weaknesses: &WeaknessesTreeType,
    rng: &mut ThreadRng,
) -> Result<(), String> {
    let choosen_types: Vec<&str> = TYPES.choose_multiple(rng, 4).cloned().collect();
    let target = choosen_types[0];
    let attackers = &choosen_types[1..];
    let (left, middle, right) = (attackers[0], attackers[1], attackers[2]);

    let target_texture = get_type_texture(target, handle, thread)?;
    let left_texture = get_type_texture(left, handle, thread)?;
    let middle_texture = get_type_texture(middle, handle, thread)?;
    let right_texture = get_type_texture(right, handle, thread)?;

    while !handle.window_should_close() {
        let mut draw_handle = handle.begin_drawing(thread);

        draw_handle.clear_background(Color::WHITE);
        draw_handle.draw_texture_rec(
            &target_texture,
            tex_rec(),
            Vector2 { x: 285.0, y: 50.0 },
            Color::WHITE,
        );

        draw_handle.draw_texture_rec(
            &left_texture,
            tex_rec(),
            Vector2 { x: 50.0, y: 280.0 },
            Color::WHITE,
        );

        draw_handle.draw_texture_rec(
            &middle_texture,
            tex_rec(),
            Vector2 { x: 285.0, y: 280.0 },
            Color::WHITE,
        );

        draw_handle.draw_texture_rec(
            &right_texture,
            tex_rec(),
            Vector2 { x: 520.0, y: 280.0 },
            Color::WHITE,
        );

        let is_left = draw_handle.is_key_pressed(KeyboardKey::KEY_LEFT)
            && handle_input(weaknesses, left, target);
        let is_middle = draw_handle.is_key_pressed(KeyboardKey::KEY_UP)
            && handle_input(weaknesses, middle, target);
        let is_right = draw_handle.is_key_pressed(KeyboardKey::KEY_RIGHT)
            && handle_input(weaknesses, right, target);

        if is_left || is_middle || is_right {
            println!("Wow you won !");
            return Ok(());
        }

        let any_keys = vec![
            draw_handle.is_key_pressed(KeyboardKey::KEY_LEFT),
            draw_handle.is_key_pressed(KeyboardKey::KEY_UP),
            draw_handle.is_key_pressed(KeyboardKey::KEY_RIGHT),
        ];

        if any_keys.contains(&true) {
            println!("You lost :/");
            return Ok(());
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let mut rng = &mut rand::thread_rng();
    set_trace_log(TraceLogType::LOG_FATAL);
    let (mut handle, thread) = raylib::init().size(640, 480).title("Hello, World").build();
    handle.set_target_fps(60);
    let weaknesses = make_graph();

    while !handle.window_should_close() {
        main_loop(&mut handle, &thread, &weaknesses, &mut rng)?;
    }
    Ok(())
}
