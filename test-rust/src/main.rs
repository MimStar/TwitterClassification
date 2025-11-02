use num_traits::NumCast;
use regex::Regex;
use csv::{Reader, StringRecord, Writer};
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::path::PathBuf;

use crate::tools::words_dictionnary_to_reg;

mod tools;

use crate::{regex_ext::RegexLogicalBuilder, rule_filter::RuleFilter};

mod regex_ext;
mod rule_filter;

fn main() -> Result<(), String>{
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