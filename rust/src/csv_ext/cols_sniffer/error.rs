use thiserror::Error;

use crate::csv_ext::cols_sniffer::AutoColumns;


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

pub fn to_auto_columns(result: &Result<AutoColumns, AutoColumnsError>, default_data: usize) -> AutoColumnsOption {
    match result {
        Ok(res) => AutoColumnsOption {
            data_column: res.data_column,
            rating_column: Some(res.rating_column)
        },
        Err(e) => e.to_auto_columns(default_data),
    }
}

impl AutoColumnsError {
    pub fn to_auto_columns(&self, default_data: usize) -> AutoColumnsOption {
        match self {
            AutoColumnsError::NoRatingFound { data_column } =>
                AutoColumnsOption {
                    data_column: *data_column,
                    rating_column: None
                },
            AutoColumnsError::NoDataFound { rating_column } =>
                AutoColumnsOption {
                    data_column: default_data,
                    rating_column: Some(*rating_column)
                },
            _ => AutoColumnsOption {data_column: default_data, rating_column: None}
        }
    }
}

pub struct AutoColumnsOption {
    pub data_column: usize,
    pub rating_column: Option<usize>,
}