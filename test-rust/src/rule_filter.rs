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

impl Hash for RuleFilter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // First, hash the variant discriminant so different variants don't collide
        std::mem::discriminant(self).hash(state);

        match self {
            RuleFilter::TRIM(s, re) => {
                s.hash(state);
                re.as_str().hash(state); // hash the regex pattern string
            }
            RuleFilter::REPLACE(s, re, replacement) => {
                s.hash(state);
                re.as_str().hash(state); 
                replacement.hash(state);
            }
            RuleFilter::DELETE(s, re) => {
                s.hash(state);
                re.as_str().hash(state);
            }
        }
    }
}


impl PartialEq for RuleFilter {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RuleFilter::TRIM(_, re_a), RuleFilter::TRIM(_, re_b)) => re_a.as_str() == re_b.as_str(),
            (RuleFilter::REPLACE(_, re_a, rep_a), RuleFilter::REPLACE(_, re_b, rep_b)) => re_a.as_str() == re_b.as_str() && rep_a == rep_b,
            (RuleFilter::DELETE(_, re_a), RuleFilter::DELETE(_, re_b)) => re_a.as_str() == re_b.as_str(),
            _ => false,
        }
    }
}
impl Eq for RuleFilter {}


impl Ord for RuleFilter {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for RuleFilter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl RuleFilter {
    pub fn name(&self) -> String {
        match self {
            RuleFilter::TRIM(name, _) => name.to_string() + " trim",
            RuleFilter::REPLACE(name, regex, _) => name.to_string() + " replacement",
            RuleFilter::DELETE(name, regex) => name.to_string() + " deletion",
        }
    }
    // Such as if there are multiple rules in a list with same pattern, delete is prioritary (since more restrictive)
    // It is a pure design choice though, not really necessary..
    fn rank(&self) -> u8 {
        match self {
            RuleFilter::DELETE(_, _) => 0,
            RuleFilter::REPLACE(_, _, _) => 1,
            RuleFilter::TRIM(_, _) => 2,
        }
    }

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