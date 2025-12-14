use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::usize;

use csv::ReaderBuilder;
use csv_sniffer::Sniffer;
use godot::global::godot_print;

use crate::csv_ext::cols_sniffer::error::AutoColumnsError;
use crate::csv_ext::transform::records_to_vec2d;

/*
If there's a header
    Check for a text/data/content/tweet section -- the data col
    Check for a rating/polarity/category/class section -- the rating col
Otherwise, pick x random entries and
    For data col, look for the col with largest averrage size, but with strictly < 280 (or 4000 ?? apparently us subs can ?)
        A little tricky since we need to check for encoding, 
    For rating, look for decimals only, where all are >=0 & <=4.
*/
pub mod error;
mod field_process;
mod sniff;
mod sniff_data;
mod sniff_rating;
mod from_header;
mod config;

pub struct ColsSniffer;

#[derive(Debug)]
pub struct AutoColumns {
    pub data_column: usize,
    pub rating_column: usize,
}

impl ColsSniffer {
    pub fn sniff_columns(path: &str) -> Result<AutoColumns, AutoColumnsError> {
        let mut sniffer = Sniffer::new();
        let mut file = File::open(path)?;
        let meta = sniffer.sniff_reader(&mut file);

        let mut rdr = ReaderBuilder::new();

        let mut has_header = false;

        if let Ok(meta) = meta {
            has_header = meta.dialect.header.has_header_row;
            let delimiter = meta.dialect.delimiter;
            let flexible = meta.dialect.flexible;

            rdr.has_headers(has_header)
                .delimiter(delimiter)
                .flexible(flexible);
        }

        
        file.seek(SeekFrom::Start(0))?;

        let mut rdr = rdr.from_reader(file);
    
        let headers = rdr.headers()?.clone();
    
        let mut veced_records = records_to_vec2d(&mut rdr.byte_records(), Some(10))?;
    
        if has_header {
            Self::sniff_columns_from_headers(&headers)
                .or_else(|err|
                    Self::sniff_columns_with_err(&mut veced_records, err))
        } else {
            Self::sniff_columns_from_vec2d(&mut veced_records)
        }
    }
}


/*
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
}*/


/* 
// Returns
// - Some(true) if it is a digit, that is, it is numeric
// - None if it is '.', ',', '+' ... that is, it might be numeric, but we need more context
// - Some(false) if it is not numeric for sure
fn is_numeric_byte(byte: u8) -> Option<bool> {
    match byte {
        b'0'..=b'9'=> Some(true),
        b'.' | b',' | b'+' | b'-' | b'e' | b'E' => None,
        _ => Some(false)
    }
}

fn is_numeric_bytes(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte|);
}*/

// Tries to figure out the columns numbers for data and rating fields in a database composed of tweets.
// 
/* 
pub fn auto_label(rdr: &mut Reader<File>, premium_sub: bool) -> Result<AutoColumns, AutoLabelError> {

}*/