use thiserror::Error;


#[derive(Error, Debug)]
pub enum AutoColumnsError {
    #[error("Csv file opened at path `{0}` is empty.")]
    CSVEmpty(String),
    #[error("An error has risen while trying to manipulate csv files - `{0}`")]
    CSVError(#[from] csv::Error),
    #[error("An error has risen while trying to open file - `{0}`")]
    IOError(#[from] std::io::Error),
    #[error("An error has risen while trying to sniff base csv informations - `{0}`")]
    SnifferError(#[from] csv_sniffer::error::SnifferError),
    #[error("No obvious field found for Rating - Data was - {data_column:?}")]
    NoRatingFound {data_column: usize},
    #[error("No obvious field found for Data - Rating was - {rating_column:?}")]
    NoDataFound {rating_column: usize},
    #[error("No obvious field found")]
    NoColumnFound,
}