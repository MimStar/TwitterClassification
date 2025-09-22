use std::{env, io, process};
use csv;

fn main() {
    let args : Vec<String> = env::args().collect();
    dbg!(&args);
    
    if let Ok(mut rdr) = csv::Reader::from_path(&args[1]) {
        for result in rdr.records() {
            if let Ok(record) = result {
                println!("{:?}", record);
            }
        }
    }
}
