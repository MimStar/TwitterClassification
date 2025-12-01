use regex::{Regex, Error};

use crate::regex_ext::builder::RegexLogicalBuilder;

pub(super) fn unvalid_emojis(positives: &[&str], negatives: &[&str]) -> Result<Regex, Error> {
    RegexLogicalBuilder::new()
            .contains(RegexLogicalBuilder::new()
                .any_of(RegexLogicalBuilder::strings_to_builders(positives))
                .one_or_more()
                .and(RegexLogicalBuilder::new()
                    .any_of(RegexLogicalBuilder::strings_to_builders(negatives))
                    .one_or_more()))
            .build()
}

pub(super) fn retweet() -> Result<Regex, Error> {
    RegexLogicalBuilder::from("RT")
        .as_whole_word()
        .build()
}

pub(super) fn url(url_headers: &[&str]) -> Result<Regex, Error> {
    RegexLogicalBuilder::new()
            .any_of(RegexLogicalBuilder::strings_to_builders(&url_headers))
            .group()
            .plus_non_space()
            .any_times()
            .as_word_end()
            .build()
}

pub(super) fn user() -> Result<Regex, Error> {
    RegexLogicalBuilder::from("@")
            .plus_non_space()
            .any_times()
            .as_whole_word()
            .build()
}