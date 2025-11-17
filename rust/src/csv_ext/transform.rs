use std::fs::File;

use csv::{ByteRecord, ByteRecordsIter, Error};

use crate::csv_ext::encoding::detect_and_decode;

pub fn records_to_vec2d(
    records: &mut ByteRecordsIter<File>,
    max_obs: Option<usize>
) -> Result<Vec<Vec<Vec<u8>>>, Error> {
    // Maps the byte records as a 2 dimensionnal array to ease exploration by columns
    // Vector of columns, that is, vector of vectors
    //      each sub vector is a column
    
    // Helper function to avoid doing an if stats.empty while iterating on records
    //          While also avoiding code dupplication !
    fn map_fields_in_vec2d(
        record: &mut ByteRecord
        , idx: usize, vec: &mut Vec<Vec<Vec<u8>>>
    ) {
        record.iter().for_each(|bytes| {
            vec[idx].push(bytes.to_vec());
        });
    }

    let mut record = records.next().unwrap()?;
    let mut vectored: Vec<Vec<Vec<u8>>> = vec![vec![]; record.len()];
    map_fields_in_vec2d(&mut record, 0, &mut vectored);

    let max_obs = max_obs.unwrap_or(usize::MAX);

    for (i, record) in records.enumerate() {
        let real_idx = i+1; // i + 1 is eww I WANT enumerate(n) SO BADLY https://github.com/rust-itertools/itertools/issues/815
        
        if max_obs <= i {break}

        let mut record = record?;
        map_fields_in_vec2d(&mut record, real_idx, &mut vectored);
    }

    Ok(vectored)

}

pub fn to_string_size(veced_records: &[Vec<Vec<u8>>]) -> Vec<Vec<usize>> {
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