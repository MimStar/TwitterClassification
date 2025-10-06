use std::{env, io, process};
use regex::Regex;
use csv::{Writer, Reader};

mod tools;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    //let positives= vec!["😀", "😄", "😆", "😁", "🥰"];
    //let negatives = vec!["😡", "😤", "😠", "🤬", "😈", "👿", "💀", "☠"];

    //let positives = vec!["a","b"];
    //let negatives = vec!["c","d"];

    let positives = tools::words_dictionnary_to_reg(args[1]);
    let negatives = tools::words_dictionnary_to_reg(args[2]);
    




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

    println!("{}", unvalid_emojis_re);

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
                    
                    let re = Regex::new(&unvalid_emojis_re).unwrap();
                    if re.is_match(truc) {
                        println!("mixed emotions : {:?}", truc);
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
                    if re.is_match(truc) {
                        test = re.replace_all(&test, "").to_string();
                        println!("url trimed : {:?}", test);
                        urls_removed += 1;
                    }

                    let re = Regex::new(&user).unwrap();
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