use regex::Regex;

use crate::regex_ext::builder::RegexLogicalBuilder;

impl RegexLogicalBuilder {
    const SPECIAL_CHARS_RE: &'static str = r"(?P<c>[\/\\\?\:\*\+\[\]\.\|\$\^=\!<>])";

    pub fn protect_string(input: &str) -> String {
        let re = Regex::new(RegexLogicalBuilder::SPECIAL_CHARS_RE).unwrap();
        let processed = re.replace_all(input, r"\$c");
        processed.into()
    }
}