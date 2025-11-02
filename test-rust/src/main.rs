use std::{borrow::Cow, collections::HashMap, env, io, process};
use regex::{Regex};
use csv::{Writer, Reader};

use crate::{regex_ext::RegexLogicalBuilder, rule_filter::RuleFilter};

mod regex_ext;
mod rule_filter;

fn main() -> Result<(), String>{
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    //let positives= vec!["😀", "😄", "😆", "😁", "🥰"];
    //let negatives = vec!["😡", "😤", "😠", "🤬", "😈", "👿", "💀", "☠"];

    //let positives = vec!["a","b"];
    //let negatives = vec!["c","d"];

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
        RuleFilter::DELETE("mixed emotions".to_string() ,unvalid_emojis_re),
        RuleFilter::DELETE("retweet".to_string(), retweet),
        RuleFilter::TRIM("url".to_string(), url),
        RuleFilter::TRIM("user".to_string(), user),
        RuleFilter::TRIM("punctuation".to_string(), punctuation)
    ];

    let mut filter_counters: HashMap<&RuleFilter, u32> = filters.iter().map(|filter| (filter, 0)).collect();

    let mut urls_removed = 0;
    let mut mixed_emotions = 0;
    let mut retweets = 0;
    let mut users_removed = 0;
    let mut punctuation_trimed = 0;

    let mut rdr = Reader::from_path(&args[1]).map_err(|e| format!("Couldn't open {} for read - {e}", &args[1]))?;

    if let Ok(mut rdr) = Reader::from_path(&args[1])
    && let Ok(mut wtr) = Writer::from_path(&args[2]) {
        for result in rdr.records() {
            if let Ok(record) = result {
                if let Some(mut truc) = record.get(5)
                && let Some(mut rating) = record.get(0) {
                    let mut processed_entry = String::from(truc);
                    for filter in &filters {
                        let mut logs = None;
                        let filtered_result = filter.apply_with_logs(&mut processed_entry, &mut logs);
                        if let Some(log_msg) = logs {
                            println!("{log_msg}");
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
            }
        }
    }

    for (filter, counter) in filter_counters {
        println!("{} : {counter}", filter.name());
    }

    Ok(())
}

fn rem_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str()
}
// Rand index