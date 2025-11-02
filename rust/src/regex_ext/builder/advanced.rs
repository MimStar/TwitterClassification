use crate::regex_ext::builder::RegexLogicalBuilder;

impl RegexLogicalBuilder {
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
}