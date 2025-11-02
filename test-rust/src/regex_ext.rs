use std::str::FromStr;
use std::ops::Add;

use regex::Regex;

#[derive(Debug)]
pub struct RegexLogicalBuilder {
    proc_result : String,   // Current state of the processed regex string
}

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

impl RegexLogicalBuilder {
    pub fn strings_to_builders(strings: &Vec<&str>) -> Vec<RegexLogicalBuilder> {
        strings.into_iter().map(|s| RegexLogicalBuilder::from(*s)).collect()
    }

    pub fn new() -> Self {
        Self::from("")
    }

    pub fn class_from(vals: Vec<&str>) -> Self {
        let mut p = String::from("[");
        for val in vals {
            p += val;
        }

        Self {proc_result : p + "]"}
    }

    pub fn contains(mut self, re: RegexLogicalBuilder) -> Self {
        self.proc_result += &(String::from(".*") + &re.proc_result);
        self
    }

    pub fn any_of(mut self, res: Vec<RegexLogicalBuilder>) -> Self {
        if res.is_empty() {
            return self;
        }

        for re in res {
            self.proc_result += &(re.group().proc_result + "|");
        }

        let mut chars = &mut self.proc_result.chars();
        chars.next_back();
        self.proc_result = String::from(chars.as_str());
        self
    }

    pub fn or(mut self) -> Self {
        self.proc_result += "|";
        self
    }

    pub fn group(mut self) -> Self {
        self.proc_result = String::from("(") + &self.proc_result + ")";
        self
    }

    pub fn plus(mut self, re: RegexLogicalBuilder) -> Self {
        self.proc_result += &re.proc_result;
        self
    }

    
    pub fn any_times(mut self) -> Self {
        self.proc_result += "*";
        self
    }
    
    pub fn one_or_more(mut self) -> Self {
        self.proc_result += "+";
        self
    }
    
    pub fn plus_non_space(mut self) -> Self {
        self.proc_result += "[^ ]";
        self
    }

    pub fn plus_anything(mut self) -> Self {
        self.proc_result += ".*";
        self
    }

    // Make sure the current processed regex is a whole word
    // that is, the whole expression is surrounded with either blank spaces, or start/end of string
    pub fn as_whole_word(mut self) -> Self {
        self.proc_result = String::from("( |^)") + &self.proc_result + "( |$)";
        self
    }

    pub fn as_word_end(mut self) -> Self {
        self.proc_result += "( |$)";
        self
    }

    // both current regex and c are present, in any order.
    // Not really the kind of things we should do in regex .. but the lab subject implies we should so ~
    pub fn and(mut self, re: RegexLogicalBuilder) -> Self {
        //self.proc_result = String::from("(?=") + &self.proc_result + ")(?=.*" + &re.proc_result + ")";
        // eeh, lookahead not supported by the regex crate
        self.proc_result = String::from("((") + &self.proc_result + ").*(" + &re.proc_result + "))|((" + &re.proc_result + ").*(" + &self.proc_result + "))";
        self
    }

    pub fn build(&self) -> Result<Regex, regex::Error> {
        Regex::new(&self.proc_result)
    }
}
