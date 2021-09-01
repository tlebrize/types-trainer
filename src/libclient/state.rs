pub enum Outcome {
    Won,
    Lost,
    Tie,
}

pub enum GameState {
    WaitingForChoices,
    GotChoices(Vec<String>, Vec<String>, usize),
    WaitingForOtherSelected,
    GotOutcome(Outcome, String, String),
}
