use std::str::FromStr;
use std::ops::Add;

use regex::Regex;

#[derive(Debug)]
pub struct RegexLogicalBuilder {
    proc_result : String,   // Current state of the processed regex string
}

mod external;
mod basics;
mod quick_hands;
mod advanced;
mod tools;
mod parse;

impl RegexLogicalBuilder {
    pub fn new() -> Self {
        Self::from("")
    }

    pub fn build(&self) -> Result<Regex, regex::Error> {
        Regex::new(&self.proc_result)
    }
}
