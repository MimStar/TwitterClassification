use num_traits::NumCast;
use regex::Regex;
use csv::{Reader, StringRecord, Writer};
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::path::PathBuf;

use crate::tools::words_dictionnary_to_reg;

mod tools;

fn main() {
    let args: Vec<String> = env::args().collect();
    naive_annotation(&args[1], &args[2], &args[3]);
}

fn naive_annotation(data_path: &str, positive_path: &str, negative_path: &str) -> Result<String, Box<dyn Error>> {
    let pos_reg = words_dictionnary_to_reg(&positive_path)?;
    let neg_reg = words_dictionnary_to_reg(&negative_path)?;

    let mut rdr = Reader::from_path(data_path)?;
    let mut wtr = Writer::from_path("naive_annotation.csv")?;

    rdr.records().into_iter().for_each(|record| {
        let ex_record =  record.unwrap();
        record_analysis(ex_record, &pos_reg, &neg_reg, &mut wtr, 5);
    });


    let path = fs::canonicalize(PathBuf::from("naive_annotation.csv"))?;
    Ok(path.display().to_string())
}

fn record_analysis(record: StringRecord, pos_reg: &Vec<String>, neg_reg: &Vec<String>, wtr: &mut Writer<File>, obj_col: usize) -> Result<(), Box<dyn Error>> {
    let obj = record.get(obj_col).ok_or(format!("No column {obj_col} found"))?;

    let mut positives: u32 = 0;
    let mut negatives: u32 = 0;
    obj.split(" ").into_iter().for_each(|mut word| {
        word = word.trim();
        if pos_reg.contains(&word.to_string()) {
            positives += 1;
        } else if neg_reg.contains(&word.to_string()) {
            negatives += 1;
        }
    });

    let rating = compute_polarity_with_weight(negatives, positives, 0_f32)?;

    wtr.write_record(&[format!("{rating}").as_str(), obj]);

    Ok(())
}

// weight is between 0 and 1
// 0 means if positives > negatives, polarity is 4, and vice versa
// 1 means polarity is 0/4 if there is exclusively negative or positive words respectively, 2 otherwise.
fn compute_polarity_with_weight(negatives: u32, positives: u32, weight: f32) -> Result<u32, Box<dyn Error>> {
    let f_negatives : f32 = NumCast::from(negatives).ok_or(format!("Cannot cast {negatives} to f32"))?;
    let f_positives : f32 = NumCast::from(positives).ok_or(format!("Cannot cast {positives} to f32"))?;
    let f_total = f_negatives + f_positives;

    //println!("{}", f_negatives);
    //println!("{}", f_positives);
    
    let pos_ratio = f_positives / f_total;
    if pos_ratio > weight || pos_ratio == 1_f32 { return Ok(4);}
    let neg_ratio = f_negatives / f_total;
    if neg_ratio > weight || neg_ratio == 1_f32 { return Ok(0);}

    Ok(2)
}

fn clean_data_body(data_path: &str) -> Result<String, &str> {
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

    let start = "(( )|^)";
    let end = "( |$)";
    let retweet = format!("{start}RT{end}");
    let url = format!("((http:)|https:|(www.))[^ ]*{end}");
    let user = format!("{start}@[^ ]*{end}");
    let punctuation = "[!\\?\\\"\\.;,\\:\\*]";
    let hashtag = format!("{start}#[^ ]*{end}");

    //println!("{}", unvalid_emojis_re);

    let mut urls_removed = 0;
    let mut mixed_emotions = 0;
    let mut retweets = 0;
    let mut users_removed = 0;
    let mut punctuation_trimed = 0;
    let mut hashtag_trimed = 0;

    if let Ok(mut rdr) = Reader::from_path(data_path)
    && let Ok(mut wtr) = Writer::from_path("clean_data_temp.csv") {
        for result in rdr.records() {
            if let Ok(record) = result {
                if let Some(mut truc) = record.get(5)
                && let Some(mut rating) = record.get(0) {
                    
                    let re = Regex::new(&unvalid_emojis_re).unwrap();
                    if re.is_match(truc) {
                        println!("mixed emotions deleted : {:?}", truc);
                        mixed_emotions += 1;
                        continue;
                    }

                    let re = Regex::new(&retweet).unwrap();
                    if re.is_match(truc) {
                        println!("retweet deleted : {:?}", truc);
                        retweets += 1;
                        continue;
                    }

                    let mut test = String::from(truc);

                    let re = Regex::new(&url).unwrap();
                    if re.is_match(&test) {
                        test = re.replace_all(&test, "").to_string();
                        println!("url trimed : {:?}", test);
                        urls_removed += 1;
                    }

                    let re = Regex::new(&user).unwrap();
                    if re.is_match(&test) {
                        test = re.replace_all(&test, "").to_string();
                        println!("user trimed : {:?}", test);
                        users_removed += 1;
                    }

                    let re = Regex::new(&punctuation).unwrap();
                    if re.is_match(&test) {
                        test = re.replace_all(&test, "").to_string();
                        println!("punctuation trimed : {:?}", test);
                        punctuation_trimed += 1;
                    }

                    let re = Regex::new(&hashtag).unwrap();
                    if re.is_match(&test) {
                        test = re.replace_all(&test, "").to_string();
                        println!("hashtag trimed : {:?}", test);
                        hashtag_trimed += 1;
                    }
                    
                    wtr.write_record(&[rating, &test]);

                }
            }
        }
        //println!("mixed emotions : {mixed_emotions}\nurls trimed : {urls_removed}\nrts deleted: {retweets}\nusers trimed: {users_removed}\npunctuation trimed: {punctuation_trimed}\nhashtag trimed : {hashtag_trimed}");
        return match fs::canonicalize(PathBuf::from("clean_data_temp.csv")) {
                        Ok(path) => Ok(path.display().to_string()),
                        Err(e) => Err("Couldn't parse output file"),
                    };
    }

    return Err("Couldn't open input/output");
}

fn rem_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str()
}
// Rand index