use godot::prelude::*;
use regex::Regex;
use csv::{Writer, Reader};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::cleandata::rule_filter::RuleFilter;
use crate::csv_ext::cols_sniffer::error::AutoColumnsError;
use crate::csv_ext::cols_sniffer::{AutoColumns, ColsSniffer};
use crate::regex_ext::builder::RegexLogicalBuilder;
//mod dirty_impl;

use crate::csv_ext::encoding;

mod rule_filter;

const DATA_COL: usize = 1;
const RATING_COL: usize = 0;

#[derive(GodotClass)]
#[class(base=Node)]
struct CleanData {
    base: Base<Node>,
}

#[godot_api]
impl INode for CleanData {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl CleanData {
    #[func]
    fn clean_data(&mut self, path: GString) -> GString {
        return match self.clean_data_body(&path.to_string()) {
            Ok(temp_path) => GString::from(temp_path),
            Err(e) => {
                self.signals().log_sent().emit(&GString::from(e));
                return GString::from("");
            },
        };
    }

    fn clean_data_generic(&mut self, input_path: &str, output_path: &str, data_col: usize, rating_col: usize, filters: &Vec<RuleFilter>) -> Result<String, String> {
        let mut filter_counters: HashMap<&RuleFilter, u32> = filters.iter().map(|filter| (filter, 0)).collect();

        let mut rdr = Reader::from_path(input_path).map_err(|e|
            format!("Couldn't open {input_path} for read - {e}"))?;
        let mut wtr = Writer::from_path(output_path).map_err(|e|
            format!("Couldn't open {output_path} for write - {e}"))?;

        for result in rdr.byte_records() { // Byte records instead, then convert

            let record = result.map_err(|e| format!("Couldn't read entry in input csv - {e}"))?;
            let tweet = record.get(data_col).ok_or(
                format!("Message column not found in record {record:?}"))?;
            let (tweet, _) = encoding::detect_and_decode(tweet);

            let rating = record.get(rating_col).ok_or(
                format!("Rating column not found in record {record:?}"))?;
            let (rating, _) = encoding::detect_and_decode(rating);
            

            let mut processed_entry = String::from(tweet);
            let mut filters_iter = filters.iter();

            loop {
                // Still filter left to apply
                if let Some(filter) = filters_iter.next() {
                    let mut logs = None;
                    let filtered_result = filter.apply_with_logs(&mut processed_entry, &mut logs);
                    if let Some(log_msg) = logs {
                        println!("{log_msg}");
                        self.signals().log_sent().emit(&GString::from(log_msg));
                        if let Some(counter) = filter_counters.get_mut(filter) {
                            *counter += 1; 
                        }
                    }

                    match filtered_result {
                        // The filters restricted the data up to the point there's no data anymore
                        // We should break without recording it then.
                        None => break,
                        Some(passed) => {
                            if let Cow::Owned(new_value) = passed {
                                processed_entry = new_value;
                            }
                        }
                    }
                } else {   // No filter left, and data remaining
                    wtr.write_record(&[&rating, &processed_entry]).map_err(|e| format!("Coudln't write the current record - {:?} : {e}", [&rating, &processed_entry]))?;
                    break;
                }
            }
        }
        
        return match fs::canonicalize(PathBuf::from(output_path)) {
                        Ok(path) => Ok(path.display().to_string()),
                        Err(e) => Err(format!("Couldn't parse output file - {e}")),
                    };
    }

    fn clean_data_body(&mut self, data_path: &str) -> Result<String, String> {
        // AUTO FILTERS GENERATION
        let positives = vec!["(:\\))", "(:\\-\\))", "(:D)"];    //
        let negatives = vec!["(:\\()", "(:\\-\\()", "(D:)"];    // These should ideally be parametrized ?

        let unvalid_emojis_re = RegexLogicalBuilder::new()
            .contains(RegexLogicalBuilder::new()
                .any_of(RegexLogicalBuilder::strings_to_builders(&positives))
                .one_or_more()
                .and(RegexLogicalBuilder::new()
                    .any_of(RegexLogicalBuilder::strings_to_builders(&negatives))
                    .one_or_more()))
            .build().unwrap();

        let retweet = RegexLogicalBuilder::from("RT").as_whole_word().build().unwrap();

        let url_list = vec!["http:", "https:", "www."];
        let url = RegexLogicalBuilder::new()
            .any_of(RegexLogicalBuilder::strings_to_builders(&url_list))
            .group()
            .plus_non_space()
            .any_times()
            .as_word_end()
            .build().unwrap();

        let user = RegexLogicalBuilder::from("@")
            .plus_non_space()
            .any_times()
            .as_whole_word()
            .build().unwrap();

        let punctuation = Regex::new("[!\\?\\\"\\.;,\\:\\*]").unwrap();

        let filters = vec![
            RuleFilter::DELETE("mixed emotio    ns".to_string() ,unvalid_emojis_re),
            RuleFilter::DELETE("retweet".to_string(), retweet),
            RuleFilter::TRIM("url".to_string(), url),
            RuleFilter::TRIM("user".to_string(), user),
            RuleFilter::TRIM("punctuation".to_string(), punctuation)
        ];

        // Warning here, rating and data cols might end up being the same
         
        let auto_columns = ColsSniffer::sniff_columns(data_path).unwrap_or_else(|err| {
            match err {
                AutoColumnsError::NoDataFound { rating_column } => AutoColumns {data_column: DATA_COL, rating_column},
                AutoColumnsError::NoRatingFound { data_column } => AutoColumns {data_column, rating_column: RATING_COL},
                _ => AutoColumns { data_column: DATA_COL, rating_column: RATING_COL },
            }
        });
        
        // CALL WITH GENERATED FILTERS AND STATIC COLUMNS
        self.clean_data_generic(
            data_path, 
            "clean_data_temp.csv", 
            auto_columns.data_column, 
            auto_columns.rating_column, 
            &filters
        )
    }

    #[signal]
    fn log_sent(message : GString);
}
