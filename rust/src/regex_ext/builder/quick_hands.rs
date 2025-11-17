use crate::regex_ext::builder::RegexLogicalBuilder;

impl RegexLogicalBuilder {
    pub fn plus_non_space(mut self) -> Self {
        self.proc_result += "[^ ]";
        self
    }

    pub fn plus_anything(mut self) -> Self {
        self.proc_result += ".*";
        self
    }

    pub fn contains(mut self, re: RegexLogicalBuilder) -> Self {
        self.proc_result += &(String::from(".*") + &re.proc_result);
        self
    }
}