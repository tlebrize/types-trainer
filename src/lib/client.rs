pub fn parse_choices(choices: String) -> (Vec<String>, Vec<String>) {
    let c: Vec<Vec<String>> = choices
        .splitn(2, ";")
        .map(|x| {
            x.splitn(2, ":")
                .nth(1)
                .unwrap()
                .split(",")
                .map(String::from)
                .collect()
        })
        .collect();
    (c[0].clone(), c[1].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_choice() {
        assert_eq!(
            parse_choices("yours:a,b,c;theirs:d,e,f".to_string()),
            (
                vec!["a".to_string(), "b".to_string(), "c".to_string(),],
                vec!["d".to_string(), "e".to_string(), "f".to_string(),]
            )
        );
    }
}
