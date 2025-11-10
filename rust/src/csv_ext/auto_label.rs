use std::fs::File;

use csv::{ByteRecordsIntoIter, ByteRecordsIter, Reader};
use thiserror::Error;

/*
If there's a header
    Check for a text/data/content/tweet section -- the data col
    Check for a rating/polarity/category/class section -- the rating col
Otherwise, pick x random entries and
    For data col, look for the col with largest averrage size, but with strictly < 280 (or 4000 ?? apparently us subs can ?)
        A little tricky since we need to check for encoding, 
    For rating, look for decimals only, where all are >=0 & <=4.
*/
pub struct AutoColumns {
    data_column: usize,
    rating_column: usize,
}

#[derive(Error, Debug)]
pub enum AutoLabelError {
    #[error("Csv file opened at path `{0}` is empty.")]
    CSVEmpty(String),
    #[error("An error has risen while trying to manipulate csv files - `{0}`")]
    CSVError(#[from] csv::Error),
    #[error("No obvious field found for Rating - Data was - {data_column:?}")]
    NoRatingFound {data_column: usize},
    #[error("No obvious field found for Data - Rating was - {rating_column:?}")]
    NoDataFound {rating_column: usize},
    #[error("No obvious field found")]
    NoLabelFound,
}


const HEADER_MAX_SIZE : usize = 255;
// Check first row is only string
//  if second row is not only strings - It's true !
//  ow - if first row strings are of a reasonnable size ? we guess it's tru
// else - False
// eeh - rebuilding the reader for each process ???
// We need to find a way to pass the reader over... but csv doesnt provide a way to get a bufferedReader
pub fn detect_header(path: String) -> Result<bool, AutoLabelError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;

    let first_row = rdr.byte_records().next();
    
    if first_row == None {
        return Err(AutoLabelError::CSVEmpty(path));
    }

    let second_row = rdr.byte_records().next();

    if Err(_) = second_row {
        return Ok(false);
    }

    let first_row_is_string = first_row.clone().iter().all(|result|
        if let Ok(field) = result {
            // ? Check they are not numbers ??
        });

    if !first_row_is_string {
        return false;
    }

    let second_row_is_string = first_row.iter().all(|result|
        if let Ok(field) = result {
            // ? Check they are not numbers ??
        } else {
            return false;
        });

    if !second_row_is_string {
        return true;
    }

    let first_row_has_probable_size = first_row.iter().all(|result|
        if let Ok(field) = result {
            return field.len() < HEADER_MAX_SIZE;
        } else {
            return false;
        });
    
    false
}

// Tries to figure out the columns numbers for data and rating fields in a database composed of tweets.
// 
pub fn auto_label(rdr: &mut Reader<File>, premium_sub: bool) -> Result<AutoColumns, AutoLabelError> {

}