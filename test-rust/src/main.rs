use std::{env, io, process};
use regex::Regex;
use csv;

fn main() {
    let args : Vec<String> = env::args().collect();
    dbg!(&args);
    
    if let Ok(mut rdr) = csv::Reader::from_path(&args[1]) {
        for result in rdr.records() {
            if let Ok(record) = result {
                if let Some(truc) = record.get(5) {
                    
                    let re = Regex::new(r".*yes.*").unwrap();
                    if re.is_match(truc) {
                        println!("{:?}", truc);
                    }
                }
            }
        }
    }
}
