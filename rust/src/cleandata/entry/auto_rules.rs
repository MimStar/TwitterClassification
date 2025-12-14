use regex::Regex;

use crate::cleandata::{entry::rules_regex, error::CleanDataError, rule_filter::RuleFilter};


static POSITIVES: [&str; 3] = ["(:\\))", "(:\\-\\))", "(:D)"];
static NEGATIVES: [&str; 3] = ["(:\\()", "(:\\-\\()", "(D:)"];    // These should ideally be parametrized ?

static ULR_HEADERS: [&str; 3] = ["http:", "https:", "www."];

pub(super) fn filters() -> Result<Vec<RuleFilter>, CleanDataError> {
    let url: Regex = rules_regex::url(&ULR_HEADERS)?;
    let unvalid_emojis_re: Regex = rules_regex::unvalid_emojis(&POSITIVES, &NEGATIVES)?;
    let punctuation: Regex = Regex::new("[!\\?\\\"\\.;,\\:\\*]")?;
    Ok(vec![
        RuleFilter::DELETE("mixed emotions".to_string(), unvalid_emojis_re),
        RuleFilter::DELETE("retweet".to_string(), rules_regex::retweet()?),
        RuleFilter::TRIM("url".to_string(), url),
        RuleFilter::TRIM("user".to_string(), rules_regex::user()?),
        RuleFilter::TRIM("punctuation".to_string(), punctuation)
    ])
}