use crate::regex_ext::builder::RegexLogicalBuilder;

impl RegexLogicalBuilder {
    pub fn class_from(vals: Vec<&str>) -> Self {
        let mut p = String::from("[");
        for val in vals {
            p += val;
        }

        Self {proc_result : p + "]"}
    }

    pub fn strings_to_builders(strings: &[&str]) -> Vec<RegexLogicalBuilder> {
        strings.into_iter().map(|s| RegexLogicalBuilder::from(*s)).collect()
    }
}