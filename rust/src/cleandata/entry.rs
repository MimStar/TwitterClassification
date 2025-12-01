use regex::Regex;

use crate::cleandata::CleanData;
use crate::cleandata::error::CleanDataError;
use crate::cleandata::rule_filter::RuleFilter;

use crate::csv_ext::cols_sniffer::{AutoColumns, ColsSniffer};
use crate::csv_ext::cols_sniffer::error::AutoColumnsError;

mod auto_rules;

const DATA_COL: usize = 1;
const RATING_COL: usize = 4;

impl CleanData {
    pub(super) fn clean_data_body(&mut self, data_path: &str) -> Result<String, CleanDataError> {
        // AUTO FILTERS GENERATION
        let punctuation = Regex::new("[!\\?\\\"\\.;,\\:\\*]")?;
        
        let positives = vec!["(:\\))", "(:\\-\\))", "(:D)"];    //
        let negatives = vec!["(:\\()", "(:\\-\\()", "(D:)"];    // These should ideally be parametrized ?
        let unvalid_emojis_re = auto_rules::unvalid_emojis(&positives, &negatives)?;

        let url_headers = vec!["http:", "https:", "www."];
        let url = auto_rules::url(&url_headers)?;

        let filters = vec![
            RuleFilter::DELETE("mixed emotions".to_string(), unvalid_emojis_re),
            RuleFilter::DELETE("retweet".to_string(), auto_rules::user()?),
            RuleFilter::TRIM("url".to_string(), url),
            RuleFilter::TRIM("user".to_string(), auto_rules::retweet()?),
            RuleFilter::TRIM("punctuation".to_string(), punctuation)
        ];

        // Warning here, rating and data cols might end up being the same
        let auto_columns = ColsSniffer::sniff_columns(data_path).unwrap_or_else(|err| {
            match err {
                AutoColumnsError::NoDataFound { rating_column } => AutoColumns {data_column: DATA_COL, rating_column},
                AutoColumnsError::NoRatingFound { data_column } => AutoColumns {data_column, rating_column: RATING_COL},
                _ => AutoColumns { data_column: DATA_COL, rating_column: RATING_COL },
            }
        });
        
        // CALL WITH GENERATED FILTERS AND STATIC COLUMNS
        self.clean_data_generic(
            data_path,
            "clean_data_temp.csv", 
            auto_columns.data_column, 
            auto_columns.rating_column, 
            &filters
        )
    }
}