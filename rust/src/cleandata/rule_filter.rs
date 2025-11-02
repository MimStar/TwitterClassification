use std::{borrow::Cow, cmp::Ordering, hash::{Hash, Hasher}};

use regex::Regex;

// First parameter is a log to display
// Second is the regex to match
// Others are specific to the filter type 
#[derive(Debug, Clone)]
pub enum RuleFilter {
    TRIM(String, Regex),            // trim matching from entry
    REPLACE(String, Regex, String), // replace matching from entry
    DELETE(String, Regex),          // delete entry if matching
}

mod external;
mod tools;

impl RuleFilter {
    pub fn apply_with_logs<'a>(&self, entry: &'a mut str, logs: &mut Option<String>) -> Option<Cow<'a, str>> {
        // Return value is Some if some part of entry passed the filters, None if the entry is dropped by the filters.
        *logs = None;
        match self {
            RuleFilter::DELETE(log_msg, re) => {
                if !re.is_match(entry) {
                    return Some(Cow::Borrowed(entry));
                }

                *logs = Some(log_msg.to_string() + " deleted : " + entry);
                return None;
            },
            RuleFilter::REPLACE(log_msg, re, rep) => {
                let replaced = re.replace_all(entry, rep);
                if let Cow::Owned(ref passed) = replaced {*logs = Some(log_msg.to_string() + " replaced : " + passed)};
                return Some(replaced);
            },
            RuleFilter::TRIM(log_msg, re) => {
                let replaced = re.replace_all(entry, "");
                if let Cow::Owned(ref passed) = replaced {*logs = Some(log_msg.to_string() + " trimed : " + passed)};
                return Some(replaced);
            }
        }
    }

    pub fn apply<'a>(&self, entry: &'a mut str) -> Option<Cow<'a, str>> {
        let mut _logs= None;
        self.apply_with_logs(entry, &mut _logs)
    }
}