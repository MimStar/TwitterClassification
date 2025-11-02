use std::{borrow::Cow, cmp::Ordering};

use regex::Regex;

#[derive(Debug)]
pub enum RuleFilter {
    TRIM(Regex),                // trim matching from entry
    REPLACE(Regex, String),     // replace matching from entry
    DELETE(Regex),              // delete entry if matching
}

impl PartialEq for RuleFilter {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RuleFilter::TRIM(re_a), RuleFilter::TRIM(re_b)) => re_a.as_str() == re_b.as_str(),
            (RuleFilter::REPLACE(re_a, rep_a), RuleFilter::REPLACE(re_b, rep_b)) => re_a.as_str() == re_b.as_str() && rep_a == rep_b,
            (RuleFilter::DELETE(re_a), RuleFilter::DELETE(re_b)) => re_a.as_str() == re_b.as_str(),
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
    // Such as if there are multiple rules in a list with same pattern, delete is prioritary (since more restrictive)
    // It is a pure design choice though, not really necessary..
    fn rank(&self) -> u8 {
        match self {
            RuleFilter::DELETE(_) => 0,
            RuleFilter::REPLACE(_, _) => 1,
            RuleFilter::TRIM(_) => 2,
        }
    }

    fn apply<'a>(&self, entry: &'a mut str, stats_tracker: Option<&mut u32>) -> Option<Cow<'a, str>> {
        // Return value is Some if some part of entry passed the filters, None if the entry is dropped by the filters.
        match self {
            RuleFilter::DELETE(re) => {
                if !re.is_match(entry) {
                    return Some(Cow::Borrowed(entry));
                }

                if let Some(tracker) = stats_tracker {*tracker += 1};
                return None;
            },
            RuleFilter::REPLACE(re, rep) => {
                let replaced = re.replace_all(entry, rep);
                if let Cow::Owned(_) = replaced && let Some(tracker) = stats_tracker {*tracker += 1;};
                return Some(replaced);
            },
            RuleFilter::TRIM(re) => {
                let replaced = re.replace_all(entry, "");
                if let Cow::Owned(_) = replaced && let Some(tracker) = stats_tracker {*tracker += 1;};
                return Some(replaced);
            }
        }
    }
}