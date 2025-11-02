use std::str::FromStr;
use std::ops::Add;

use crate::regex_ext::builder::RegexLogicalBuilder;

impl From<&RegexLogicalBuilder> for String {
    fn from(s: &RegexLogicalBuilder) -> String {
        s.proc_result.clone()
    }
}

impl From<&str> for RegexLogicalBuilder {
    fn from(s: &str) -> RegexLogicalBuilder {
        Self { proc_result : String::from(s)}
    }
}

impl From<String> for RegexLogicalBuilder {
    fn from(s: String) -> RegexLogicalBuilder {
        Self { proc_result : s}
    }
}

impl Add for RegexLogicalBuilder {
    type Output = RegexLogicalBuilder;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.proc_result + &rhs.proc_result)
    }
}

impl Add<String> for RegexLogicalBuilder {
    type Output = RegexLogicalBuilder;

    fn add(self, rhs: String) -> Self::Output {
        Self::from(self.proc_result + &rhs)
    }
}