use godot::prelude::*;
use regex::Regex;
use csv::{Writer, Reader};
use core::hash;
use std::fs;
use std::path::PathBuf;
//mod dirty_impl;


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

    fn clean_data_body(&mut self, data_path: &str) -> Result<String, &str> {
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
                            //println!("mixed emotions : {:?}", truc);
                            self.signals().log_sent().emit(&GString::from(format!("mixed emotions deleted : {:?}", truc)));
                            mixed_emotions += 1;
                            continue;
                        }

                        let re = Regex::new(&retweet).unwrap();
                        if re.is_match(truc) {
                            //println!("retweet deleted : {:?}", truc);
                            self.signals().log_sent().emit(&GString::from(format!("retweet deleted : {:?}", truc)));
                            retweets += 1;
                            continue;
                        }

                        let mut test = String::from(truc);

                        let re = Regex::new(&url).unwrap();
                        if re.is_match(&test) {
                            test = re.replace_all(&test, "").to_string();
                            self.signals().log_sent().emit(&GString::from(format!("url trimed : {:?}", test)));
                            //println!("url trimed : {:?}", test);
                            urls_removed += 1;
                        }

                        let re = Regex::new(&user).unwrap();
                        if re.is_match(&test) {
                            test = re.replace_all(&test, "").to_string();
                            self.signals().log_sent().emit(&GString::from(format!("user trimed : {:?}", test)));
                            //println!("user trimed : {:?}", test);
                            users_removed += 1;
                        }

                        let re = Regex::new(&punctuation).unwrap();
                        if re.is_match(&test) {
                            test = re.replace_all(&test, "").to_string();
                            self.signals().log_sent().emit(&GString::from(format!("punctuation trimed : {:?}", test)));
                            //println!("punctuation trimed : {:?}", test);
                            punctuation_trimed += 1;
                        }

                        let re = Regex::new(&hashtag).unwrap();
                        if re.is_match(&test) {
                            test = re.replace_all(&test, "").to_string();
                            self.signals().log_sent().emit(&GString::from(format!("hashtag trimed : {:?}", test)));
                            //println!("hashtag trimed : {:?}", test);
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

    #[signal]
    fn log_sent(message : GString);
}

fn rem_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str()
}
// # !!!
