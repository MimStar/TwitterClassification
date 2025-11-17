use crate::csv_ext::label_sniffer::error::AutoLabelError;
use crate::csv_ext::label_sniffer::{AutoColumns, LabelSniffer};
use crate::csv_ext::transform::to_string_size;
use crate::csv_ext::label_sniffer::config;

impl LabelSniffer {
    pub(super) fn sniff_data_with_rating(
        veced_records: &mut [Vec<Vec<u8>>],
        rating_column: usize
    ) -> Result<AutoColumns, AutoLabelError> {
        match Self::sniff_data(veced_records) {
            Some(data_column) =>
                Ok(AutoColumns { data_column, rating_column }),
            
            None =>
                Err(AutoLabelError::NoDataFound { rating_column }),
        }
    }

    pub(super) fn sniff_data(
        veced_records: &mut [Vec<Vec<u8>>]
    ) -> Option<usize> {
        let sizes = to_string_size(veced_records);
        Self::sniff_data_from_sizes(sizes)
    }

    // Tries to infer which column contains the tweets using the length of its content.
    // If all column shows evidences that it can't be containing valid tweets, it returns None
    //      - (that is, if some messages in them are too big to be tweets)
    fn sniff_data_from_sizes(sizes: Vec<Vec<usize>>) -> Option<usize> {
        let mut best_col: Option<usize> = None;
        let mut best_score = usize::MAX; // Best score is the lowest - 
    
        for (i, lengths) in sizes.iter().enumerate() {
            let rows = lengths.len();
            let score = lengths
                .iter()
                .copied()
                .try_fold(0, |acc, length| {
                    // Early return an error if any cell is bigger than a tweet
                    if length > config::TWEET_MAX_CHARS {Err(())}
                    else {Ok(acc + length)}
                })
                .and_then(|sum| Ok(sum/rows))
                .and_then(|avg| Ok(config::TWEET_MAX_CHARS - avg))
                .unwrap_or(usize::MAX); 
                // Either the averrage, or usize::MAX if any thing was too big to be a tweet
    
            if score < best_score {
                best_score = score;
                best_col = Some(i);
            }
        }
    
        best_col
    }
}