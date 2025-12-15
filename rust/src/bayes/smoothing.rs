#[derive(Debug, Clone, Copy)]
pub enum VoteType {
    Laplace,
    Lidstone,
    AddAlpha(f64),
}

impl From<VoteType> for f64 {
    fn from(value: VoteType) -> Self {
        match value {
            VoteType::Laplace => 1.,
            VoteType::Lidstone => 0.5,
            VoteType::AddAlpha(alpha) => alpha,
        }
    }
}

impl From<i64> for VoteType {
    fn from(value: i64) -> Self {
        match value {
            1 => VoteType::Lidstone,
            _ => VoteType::Laplace,
        }
    }
}