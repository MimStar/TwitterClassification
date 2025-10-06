use std::io::BufReader;
use std::fs::File;

use regex::Regex;

pub fn words_dictionnary_to_reg(path: String) -> Result<Regex, Error> {
    let f = File::open(path)?;
    let mut rdr = BufReader::new(f);
    
    let mut re_string: String = "";

    let mut buf: String;
    rdr.split(',').for_each(move |word| {
        re_string += word.trim();
    });

    return Regex::from(re_string);
}


/*
pub fn words_dictionnary_to_vec(path: String, content_column: usize, polarity_column: usize) -> Result<Vec<String>, Error> {
    if let Ok(mut rdr) = Reader::from_path(path)
    && let Ok(mut wtr) = Writer::from_path(path) {
        rdr.records().into_iter().for_each(move |record| {
            let content =  record?.get(content_column)?;
            let re = Regex::new()
        });
    }
}*/