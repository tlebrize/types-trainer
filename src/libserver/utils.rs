#![allow(dead_code)]
use std::collections::BTreeMap;

pub const TYPES: [&str; 18] = [
    "bug", "dark", "dragon", "electric", "fairy", "fighting", "fire", "flying", "ghost", "grass",
    "ground", "ice", "normal", "poison", "psychic", "rock", "steel", "water",
];

pub type TreeType = BTreeMap<&'static str, Vec<&'static str>>;

pub fn compute_scores(
    p1_selected: String,
    p2_selected: String,
    strengths: TreeType,
    weaknesses: TreeType,
) -> (i16, i16) {
    let mut p1_score = 4;
    let mut p2_score = 4;

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
    (p1_score, p2_score)
}

pub fn make_strengths_graph() -> TreeType {
    let mut w = TreeType::new();
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

pub fn make_weaknesses_graph() -> TreeType {
    let mut w = TreeType::new();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_scores() {
        let mut s = TreeType::new();
        s.insert("bug", vec!["grass"]);
        s.insert("grass", vec!["rock"]);
        s.insert("normal", vec![]);
        s.insert("rock", vec!["bug"]);

        let mut w = TreeType::new();
        w.insert("grass", vec!["bug"]);
        w.insert("bug", vec![]);
        w.insert("normal", vec!["rock"]);
        w.insert("rock", vec!["grass"]);

        assert_eq!(
            compute_scores("bug".to_string(), "grass".to_string(), s.clone(), w.clone()),
            (8, 2)
        );

        assert_eq!(
            compute_scores(
                "normal".to_string(),
                "rock".to_string(),
                s.clone(),
                w.clone()
            ),
            (2, 4)
        );

        assert_eq!(
            compute_scores("normal".to_string(), "normal".to_string(), s, w),
            (4, 4)
        );
    }
}
