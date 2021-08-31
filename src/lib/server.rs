#![allow(dead_code)]
use std::collections::BTreeMap;

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
