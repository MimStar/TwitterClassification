use csv::ByteRecord;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CleanDataError {
    #[error("An error has risen while trying to manipulate csv files - `{0}`")]
    CSVError(#[from] csv::Error),
    #[error("An error has risen while trying to open file - `{0}`")]
    IOError(#[from] std::io::Error),
    #[error("Message column not found in record `{0:?}`")]
    MissingMessage(ByteRecord),
    #[error("Rating column not found in record `{0:?}`")]
    MissingRating(ByteRecord),
    #[error("Error while evaluating regex expression - `{0}`")]
    RegexError(#[from] regex::Error)
}