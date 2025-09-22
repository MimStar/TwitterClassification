use std::{env, io, process};
use regex::Regex;
use csv;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    //let positives= vec!["😀", "😄", "😆", "😁", "🥰"];
    //let negatives = vec!["😡", "😤", "😠", "🤬", "😈", "👿", "💀", "☠"];

    //let positives = vec!["a","b"];
    //let negatives = vec!["c","d"];

    let positives = vec!["(:\\))", "(:\\-\\))", "(:D)"];
    let negatives = vec!["(:\\()", "(:\\-\\()", "(D:)"];

    let mut positive_re = "(".to_string(); 
    for positive in positives.iter() {
        positive_re += positive;
        positive_re += "|";
    }

    positive_re = rem_last(&positive_re).to_string();
    positive_re += ")";

    let mut negative_re = "(".to_string();

    for negative in negatives.iter() {
        negative_re += negative;
        negative_re += "|";
    }

    negative_re = rem_last(&negative_re).to_string();
    negative_re += ")";

    let unvalid_positive_first_re = ".*".to_owned() + &positive_re + "+.*" + &negative_re + "+.*";
    let unvalid_negative_first_re = ".*".to_owned() + &negative_re + "+.*" + &positive_re + "+.*";
    let unvalid_emojis_re = "(".to_owned() + &unvalid_positive_first_re + ")|(" + &unvalid_negative_first_re + ")";

    println!("{}", unvalid_emojis_re);

    
    if let Ok(mut rdr) = csv::Reader::from_path(&args[1]) {
        for result in rdr.records() {
            if let Ok(record) = result {
                if let Some(truc) = record.get(5) {
                    
                    let re = Regex::new(&unvalid_emojis_re).unwrap();
                    if re.is_match(truc) {
                        println!("{:?}", truc);
                    }
                }
            }
        }
    }
}

fn rem_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str()
}
