use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::usize;

use csv::{ByteRecord, ByteRecordsIntoIter, ByteRecordsIter, Reader, ReaderBuilder, StringRecord};
use csv_sniffer::Sniffer;
use thiserror::Error;

use crate::csv_ext::encoding::detect_and_decode;

/*
If there's a header
    Check for a text/data/content/tweet section -- the data col
    Check for a rating/polarity/category/class section -- the rating col
Otherwise, pick x random entries and
    For data col, look for the col with largest averrage size, but with strictly < 280 (or 4000 ?? apparently us subs can ?)
        A little tricky since we need to check for encoding, 
    For rating, look for decimals only, where all are >=0 & <=4.
*/

static DATA_TARGET_HEADERS: &[&str] = &["tweet", "message", "content", "data", "tweet_content"];
static RATING_TARGET_HEADERS: &[&str] = &["rating", "polarity", "grade", "positivity"];
static TWEET_MAX_CHARS: usize = 280;

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
    #[error("An error has risen while trying to open file - `{0}`")]
    IOError(#[from] std::io::Error),
    #[error("An error has risen while trying to sniff base csv informations - `{0}`")]
    SnifferError(#[from] csv_sniffer::error::SnifferError),
    #[error("No obvious field found for Rating - Data was - {data_column:?}")]
    NoRatingFound {data_column: usize},
    #[error("No obvious field found for Data - Rating was - {rating_column:?}")]
    NoDataFound {rating_column: usize},
    #[error("No obvious field found")]
    NoLabelFound,
}


pub fn sniff_labels(path: String) -> Result<AutoColumns, AutoLabelError> {
    let mut sniffer = Sniffer::new();
    let mut file = File::open(path)?;
    let meta = sniffer.sniff_reader(&mut file)?;

    let has_header = meta.dialect.header.has_header_row;
    let delimiter = meta.dialect.delimiter;
    let flexible = meta.dialect.flexible;

    file.seek(SeekFrom::Start(0))?;

    let mut rdr = ReaderBuilder::new()
        .has_headers(has_header)
        .delimiter(delimiter)
        .flexible(flexible)
        .from_reader(file);

    let headers = rdr.headers()?.clone();

    let mut veced_records = map_records_to_vec(&mut rdr.byte_records(), Some(10))?;

    if has_header {
        sniff_labels_from_headers(&headers)
            .or_else(|err|
                sniff_labels_with_err(&mut veced_records, err))
    } else {
        sniff_labels_from_veced(&mut veced_records)
    }
}


pub fn map_records_to_vec(records: &mut ByteRecordsIter<File>, max_obs: Option<usize>) -> Result<Vec<Vec<Vec<u8>>>, AutoLabelError> {
    // Maps the byte records as a 2 dimensionnal array to ease exploration by columns
    // Vector of columns, that is, vector of vectors
    //      each sub vector is a column
    
    // Helper function to avoid doing an if stats.empty while iterating on records
    //          While also avoiding code dupplication !
    fn map_record_in_vec(record: &mut ByteRecord, idx: usize, vec: &mut Vec<Vec<Vec<u8>>>) {
        record.iter().for_each(|bytes| {
            vec[idx].push(bytes.to_vec());
        });
    }

    let mut record = records.next().unwrap()?;
    let mut vectored: Vec<Vec<Vec<u8>>> = vec![vec![]; record.len()];
    map_record_in_vec(&mut record, 0, &mut vectored);

    let max_obs = max_obs.unwrap_or(usize::MAX);

    for (i, record) in records.enumerate() {
        let real_idx = i+1; // i + 1 is eww I WANT enumerate(n) SO BADLY https://github.com/rust-itertools/itertools/issues/815
        
        if max_obs <= i {break}

        let mut record = record?;
        map_record_in_vec(&mut record, real_idx, &mut vectored);
    }

    Ok(vectored)

}


/* Could be interesting not to clone, let's see later how i can avoid the ownership error
the error is that vectored references bytes, which is owned by record, which is a local variable
records.next() produces an owned variable, not a reference to a record.

How can we get a reference instead ? it seems impossible by using csv::Reader from what I read of the doc
should implement my own buffer through csv-core, eh

pub fn map_records_to_vec<'a>(records: &'a mut ByteRecordsIter<File>, max_obs: Option<usize>) -> Result<Vec<Vec<&'a [u8]>>, Box<dyn std::error::Error>> {
    fn map_record_in_vec<'a>(record: &'a mut ByteRecord, idx: usize, vec: &mut Vec<Vec<&'a [u8]>>) {
        record.iter().for_each(|bytes| {
            vec[idx].push(bytes);
        });
    }

    let mut record = records.by_ref().next().unwrap()?;
    let mut vectored: Vec<Vec<&[u8]>> = vec![vec![]; record.len()];
    map_record_in_vec(&mut record, 0, &mut vectored);


    Ok(vectored)

}
*/ 

pub fn map_to_string_size(veced_records: &[Vec<Vec<u8>>]) -> Vec<Vec<usize>> {
    // Maps a 2D vector of bytes to the size of the corresponding string (according to its guessed encoding)
    // Vector of columns, that is, vector of vectors
    //      each sub vector is a column
    veced_records
        .iter()
        .map(|column| {
            column
                .iter()
                .map(|bytes| {
                    let (text, _) = detect_and_decode(bytes);
                    text.len()
                })
                .collect()
        })
        .collect()
}

// Tries to infer which column contains the rating.
// That is, a column where all content is always either 0, 2 or 4.
pub fn infer_rating_col_from_veced(records: &[Vec<Vec<u8>>]) -> Option<usize> {
    records
        .iter()
        .enumerate()
        .find(|(_, column)| {
            column
                .iter()
                .all(|field| {
                    bytes_is_rating(field)
                })
        })
        .and_then(|(i, _)| Some(i))
}

fn bytes_is_rating(bytes: &[u8]) -> bool {
    let mut bytes_iter = bytes.iter();
    
    // Can't be empty
    let first_byte = match bytes_iter.next() {
        Some(b) => b,
        None => return false,
    };

    // It can be a single byte not surrounded by quotes
    let second_byte = match bytes_iter.next() {
        Some(b) => b,
        None => return byte_is_rating(*first_byte),
    };
    

    // If there is more than one byte, there should be exactly 3 - [",x,"]
    let third_byte = match bytes_iter.next() {
        Some(b) => b,
        None => return false,
    };
    
    if let Some(_) = bytes_iter.next() {return false;}

    // check the three bytes are of format [",x,"]
    return *first_byte == b'"' && byte_is_rating(*second_byte) && *third_byte == b'"';
}

fn byte_is_rating(byte: u8) -> bool {
    match byte {
        b'0' | b'2' | b'3' => true,
        _ => false,
    }
}

// Tries to infer which column contains the tweets using the length of its content.
// If all column shows evidences that it can't be containing valid tweets, it returns None
//      - (that is, if some messages in them are too big to be tweets)
pub fn infer_data_col_from_sizes(stats: Vec<Vec<usize>>) -> Option<usize> {
    let mut best_col: Option<usize> = None;
    let mut best_score = usize::MAX; // Best score is the lowest - 

    for (i, lengths) in stats.iter().enumerate() {
        let rows = lengths.len();
        let score = lengths
            .iter()
            .copied()
            .try_fold(0, |acc, length| {
                // Early return an error if any cell is bigger than a tweet
                if length > TWEET_MAX_CHARS {
                    Err(())
                } else {
                    Ok(acc + length)
                }
            })
            .and_then(|sum| Ok(sum/rows))
            .and_then(|avg| Ok(TWEET_MAX_CHARS - avg))
            .unwrap_or(usize::MAX); 
            // Either the averrage, or usize::MAX if any thing was too big to be a tweet

        if score < best_score {
            best_score = score;
            best_col = Some(i);
        }
    }

    best_col
}

pub fn sniff_labels_with_err(veced_records: &mut [Vec<Vec<u8>>], error: AutoLabelError) -> Result<AutoColumns, AutoLabelError> {
    match error {
        AutoLabelError::NoRatingFound { data_column } => sniff_rating_with_data(veced_records, data_column),
        AutoLabelError::NoDataFound { rating_column } => sniff_data_with_rating(veced_records, rating_column),
        _ => sniff_labels_from_veced(veced_records),
    }
}

pub fn sniff_labels_from_veced(veced_records: &mut [Vec<Vec<u8>>]) -> Result<AutoColumns, AutoLabelError> {
    let data_column = infer_data_col_from_veced(veced_records);
    let rating_column = infer_rating_col_from_veced(veced_records);

    if let Some(data_column) = data_column
    && let Some(rating_column) = rating_column {
        return Ok(AutoColumns { data_column, rating_column });
    }

    if let Some(data_column) = data_column {
        return Err(AutoLabelError::NoRatingFound { data_column });
    }

    if let Some(rating_column) = rating_column {
        return Err(AutoLabelError::NoDataFound { rating_column });
    }

    return Err(AutoLabelError::NoLabelFound);
}

fn infer_data_col_from_veced(veced_records: &mut [Vec<Vec<u8>>]) -> Option<usize> {
    let sizes = map_to_string_size(veced_records);
    infer_data_col_from_sizes(sizes)
}

fn sniff_data_with_rating(veced_records: &mut [Vec<Vec<u8>>], rating_column: usize) -> Result<AutoColumns, AutoLabelError> {
    match infer_data_col_from_veced(veced_records) {
        Some(data_column) => Ok(AutoColumns { data_column, rating_column }),
        None => Err(AutoLabelError::NoDataFound { rating_column }),
    }
}

fn sniff_rating_with_data(veced_records: &mut [Vec<Vec<u8>>], data_column: usize) -> Result<AutoColumns, AutoLabelError> {
    match infer_rating_col_from_veced(&veced_records) {
        Some(rating_column) => Ok(AutoColumns {data_column, rating_column}),
        None => Err(AutoLabelError::NoRatingFound { data_column }),
    }
}


pub fn sniff_labels_from_headers(headers: &StringRecord) -> Result<AutoColumns, AutoLabelError> {
    let mut cols = AutoColumns {data_column: 0, rating_column: 0};
    let mut data_found = false;
    let mut rating_found = false;

    headers
        .iter()
        .enumerate()
        .for_each(|(i, header)| {
            let lower_header = header.to_lowercase();
            
            if DATA_TARGET_HEADERS.contains(&lower_header.as_str()) {
                cols.data_column = i;
                if rating_found {
                    return;
                }
                data_found = true;
            } else if RATING_TARGET_HEADERS.contains(&lower_header.as_str()) {
                cols.rating_column = i;
                if data_found {
                    return;
                }
                rating_found = true;
            }
        });
    
    if data_found && rating_found {
        return Ok(cols);
    }

    if data_found {
        return Err(AutoLabelError::NoRatingFound { data_column: cols.data_column })
    }

    if rating_found {
        return Err(AutoLabelError::NoDataFound { rating_column: cols.rating_column })
    }

    return Err(AutoLabelError::NoLabelFound);
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