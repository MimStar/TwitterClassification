use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Representation {
    Presence,
    Frequence,
}

impl Representation {
    pub fn tokens_to_count(&self, tokens: Vec<String>) -> Vec<String> {
        match self {
            Representation::Presence => {
                let unique: HashSet<_> = tokens.into_iter().collect();
                unique.into_iter().collect()
            },
            Representation::Frequence => tokens
        }
    }
}

impl From<i64> for Representation {
    fn from(value: i64) -> Self {
        match value as usize {
            1 => Representation::Frequence,
            _ => Representation::Presence,
        }
    }
}