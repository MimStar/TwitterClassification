use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use csv::{Reader, Writer};

use godot::builtin::GString;
use godot::obj::WithUserSignals;
use itertools::Itertools;

use crate::cleandata::CleanData;
use crate::cleandata::error::CleanDataError;
use crate::cleandata::rule_filter::RuleFilter;

use crate::csv_ext::encoding;

impl CleanData {
    pub(super) fn clean_data_generic(
        &mut self, input_path: &str,
        output_path: &str,
        data_col: usize,
        rating_col: usize,
        filters: &Vec<RuleFilter>
    ) -> Result<String, CleanDataError>
    {
        let mut filter_counters: HashMap<&RuleFilter, u32> =
            filters
                .iter()
                .map(|filter| (filter, 0))
                .collect();     // To track the number of tweets that pass through each filters

        let mut rdr = Reader::from_path(input_path)?;
        let mut wtr = Writer::from_path(output_path)?;
        let mut saved_records: Vec<[String; 2]> = vec![];

        // Using byte records since it is not necessarily utf-8
        // We want to be flexible over the encoding format of the csv entries, so we'll decode them manually.
        for result in rdr.byte_records() {

            let record = result?;
            let tweet = record
                .get(data_col)
                .ok_or(CleanDataError::MissingMessage(record.clone()))?;

            let (tweet, _) = encoding::detect_and_decode(tweet);


            let rating = record
                .get(rating_col)
                .ok_or(CleanDataError::MissingRating(record.clone()))?;

            let (rating, _) = encoding::detect_and_decode(rating);
            

            let mut processed_entry = String::from(tweet);
            let mut filters_iter = filters.iter();

            loop {
                // Still some filters left to apply
                if let Some(filter) = filters_iter.next() {
                    let mut logs = None;
                    let filtered_result = filter.apply_with_logs(&mut processed_entry, &mut logs);

                    if let Some(log_msg) = logs {
                        self.signals()
                            .log_sent()
                            .emit(&GString::from(log_msg));

                        if let Some(counter) = filter_counters.get_mut(filter) {
                            *counter += 1; 
                        }
                    }

                    match filtered_result {
                        // Entering here means the filters trimmed the data 
                        //  up to the point that it is empty.
                        // We should thus move to the next record, and drop this one.
                        // - dropping means not recording it in the output csv.
                        None => break,
                        Some(passed) => {
                            if let Cow::Owned(new_value) = passed {
                                processed_entry = new_value;
                            }
                        }
                    }
                } else {   // No filter left to apply, and some data is remaining
                    saved_records.push([rating, processed_entry]);
                    break;
                }
            }
        }

        let prev_size = saved_records.len();
        let uniqued_records = saved_records
            .iter()
            .unique_by(|entry| &entry[1])
            .collect::<Vec<_>>();

        self.signals()
            .log_sent()
            .emit(&GString::from(
                format!("Removed {} dupplicates.", prev_size - uniqued_records.len())
            ));

        for record in uniqued_records {
            wtr.write_record(record)?;
        }

        let path = fs::canonicalize(PathBuf::from(output_path))?;
        Ok(path.display().to_string())
    }
}