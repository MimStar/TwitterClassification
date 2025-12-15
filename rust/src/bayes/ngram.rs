#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NgramMode {
    Uni,
    Bi,
    UniBi,
}

impl NgramMode {
    pub fn tokeniser_tweet(&self, tweet: &str) -> Vec<String> {
        let unigrams: Vec<String> = tweet.split_whitespace()
            .map(|w| w.to_lowercase())
            .filter(|w| w.chars().count() >= 4)
            .collect();

        let mut tokens = Vec::new();

        match self {
            NgramMode::Uni => {
                tokens = unigrams;
            },
            NgramMode::Bi => {
                for window in unigrams.windows(2) {
                    tokens.push(format!("{} {}", window[0], window[1]));
                }
            },
            NgramMode::UniBi => {
                tokens.extend(unigrams.clone());
                for window in unigrams.windows(2) {
                    tokens.push(format!("{} {}", window[0], window[1]));
                }
            }
        }
        
        tokens
    }
}

impl From<i64> for NgramMode {
    fn from(value: i64) -> Self {
        match value {
            1 => NgramMode::Bi,
            2 => NgramMode::UniBi,
            _ => NgramMode::Uni,
        }
    }
}