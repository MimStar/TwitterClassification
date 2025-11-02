use crate::cleandata::rule_filter::RuleFilter;



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
    pub(super) fn rank(&self) -> u8 {
        match self {
            RuleFilter::DELETE(_, _) => 0,
            RuleFilter::REPLACE(_, _, _) => 1,
            RuleFilter::TRIM(_, _) => 2,
        }
    }
}