use std::{env, io, process};
use regex::{Regex};
use csv::{Writer, Reader};

use crate::regex_ext::RegexLogicalBuilder;

mod regex_ext;

fn main() {
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
                                            .one_or_more()));


    let start = "(( )|^)";
    let end = "( |$)";

    let retweet = RegexLogicalBuilder::from("RT").as_whole_word();

    //let retweet = format!("{start}RT{end}");
    //let url = format!("((http:)|https:|(www.))[^ ]*{end}");
    let url_list = vec!["http:", "https:", "www."];

    let url = RegexLogicalBuilder::new()
                    .any_of(RegexLogicalBuilder::strings_to_builders(&url_list))
                    .group()
                    .plus_non_space()
                    .any_times()
                    .as_word_end();

    //let user = format!("{start}@[^ ]*{end}");

    let user = RegexLogicalBuilder::from("@")
                    .plus_non_space()
                    .any_times()
                    .as_whole_word();

    let punctuation = "[!\\?\\\"\\.;,\\:\\*]";

    let str : String = (&unvalid_emojis_re).into();
    //let str = unvalid_emojis_re;
    //println!("{}", &unvalid_emojis_re);

    let mut urls_removed = 0;
    let mut mixed_emotions = 0;
    let mut retweets = 0;
    let mut users_removed = 0;
    let mut punctuation_trimed = 0;

    if let Ok(mut rdr) = Reader::from_path(&args[1])
    && let Ok(mut wtr) = Writer::from_path(&args[2]) {
        for result in rdr.records() {
            if let Ok(record) = result {
                if let Some(mut truc) = record.get(5)
                && let Some(mut rating) = record.get(0) {
                    
                    //let re = Regex::new(&unvalid_emojis_re).unwrap();
                    let re = unvalid_emojis_re.build().unwrap();
                    if re.is_match(truc) {
                        println!("mixed emotions : {:?}", truc);
                        mixed_emotions += 1;
                        continue;
                    }

                    //let re = Regex::new(&retweet).unwrap();
                    let re = retweet.build().unwrap();
                    if re.is_match(truc) {
                        println!("retweet deleted : {:?}", truc);
                        retweets += 1;
                        continue;
                    }

                    let mut test = String::from(truc);

                    //let re = Regex::new(&url).unwrap();
                    let re = url.build().unwrap();
                    if re.is_match(truc) {
                        test = re.replace_all(&test, "").to_string();
                        println!("url trimed : {:?}", test);
                        urls_removed += 1;
                    }

                    //let re = Regex::new(&user).unwrap();
                    let re = user.build().unwrap();
                    if re.is_match(truc) {
                        test = re.replace_all(&test, "").to_string();
                        println!("user trimed : {:?}", test);
                        users_removed += 1;
                    }

                    let re = Regex::new(&punctuation).unwrap();
                    if re.is_match(punctuation) {
                        test = re.replace_all(&test, "").to_string();
                        println!("punctuation trimed : {:?}", test);
                        punctuation_trimed += 1;
                    }

                    wtr.write_record(&[rating, &test]);
                }
            }
        }
    }
    println!("mixed emotions : {mixed_emotions}\nurls trimed : {urls_removed}\nrts deleted: {retweets}\nusers trimed: {users_removed}\npunctuation trimed: {punctuation_trimed}");
}

fn rem_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str()
}
// Rand index