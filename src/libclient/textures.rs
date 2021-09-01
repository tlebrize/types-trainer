use raylib::prelude::*;
use std::collections::BTreeMap;

fn get_type_texture(
    type_: &str,
    handle: &mut raylib::RaylibHandle,
    thread: &RaylibThread,
) -> Result<Texture2D, String> {
    let type_filename = format!("media/{}.png", type_);
    let type_image = Image::load_image(&type_filename)?;
    handle.load_texture_from_image(thread, &type_image)
}

pub struct TextureStore {
    pub textures: BTreeMap<&'static str, Texture2D>,
}

impl TextureStore {
    pub fn new(handle: &mut RaylibHandle, thread: &RaylibThread) -> Self {
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
