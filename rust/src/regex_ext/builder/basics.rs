use crate::regex_ext::builder::RegexLogicalBuilder;

impl RegexLogicalBuilder {
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
}