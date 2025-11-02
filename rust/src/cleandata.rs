use godot::prelude::*;
use regex::Regex;
use csv::{Writer, Reader};
use core::hash;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::cleandata::rule_filter::RuleFilter;
use crate::regex_ext::builder::RegexLogicalBuilder;
//mod dirty_impl;

mod rule_filter;

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
            Err(e) => GString::from(e),
        };
    }

    fn clean_data_body(&mut self, data_path: &str) -> Result<String, String> {
        let positives = vec!["(:\\))", "(:\\-\\))", "(:D)"];
        let negatives = vec!["(:\\()", "(:\\-\\()", "(D:)"];

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

        let mut filter_counters: HashMap<&RuleFilter, u32> = filters.iter().map(|filter| (filter, 0)).collect();

        let mut rdr = Reader::from_path(data_path).map_err(|e|
            format!("Couldn't open {data_path} for read - {e}"))?;
        let mut wtr = Writer::from_path("clean_data_temp.csv").map_err(|e|
            format!("Couldn't open clean_data_temp.csv for write - {e}"))?;

        for result in rdr.records() {
            let record = result.map_err(|e| format!("Couldn't read entry in input csv - {e}"))?;
            let tweet = record.get(5).ok_or(
                format!("Message column not found in record {record:?}"))?;

            let rating = record.get(0).ok_or(
                format!("Rating column not found in record {record:?}"))?;

            let mut processed_entry = String::from(tweet);
            for filter in &filters {
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
                    None => break,
                    Some(passed) => {
                        if let Cow::Owned(new_value) = passed {
                            processed_entry = new_value;
                        }
                    }
                }
            }

            wtr.write_record(&[rating, &processed_entry]);
        }
        
        return match fs::canonicalize(PathBuf::from("clean_data_temp.csv")) {
                        Ok(path) => Ok(path.display().to_string()),
                        Err(e) => Err("Couldn't parse output file".to_string()),
                    };
    }

    #[signal]
    fn log_sent(message : GString);
}

fn rem_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str()
}
// # !!!
