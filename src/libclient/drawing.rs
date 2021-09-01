use crate::Outcome;
use crate::TextureStore;
use raylib::prelude::*;

fn tex_rec() -> Rectangle {
    Rectangle {
        x: 0.0,
        y: 0.0,
        width: 70.0,
        height: 70.0,
    }
}

pub fn retry(draw_handle: &mut RaylibDrawHandle) -> bool {
    draw_handle.draw_text("Press enter to play again.", 10, 10, 10, Color::BLACK);
    draw_handle.is_key_pressed(KeyboardKey::KEY_ENTER)
}

pub fn draw_outcome(
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

pub fn draw_choices(
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
