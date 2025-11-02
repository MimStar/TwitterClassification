use std::{cmp::Ordering, hash::{Hash, Hasher}};

use crate::cleandata::rule_filter::RuleFilter;


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