use godot::prelude::*;

mod generic;
mod entry;
mod error;
mod rule_filter;

#[derive(GodotClass)]
#[class(base=Node)]
struct CleanData {
    base: Base<Node>,
}

#[godot_api]
impl INode for CleanData {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl CleanData {
    #[func]
    fn clean_data(&mut self, path: GString) -> GString {
        return match self.clean_data_body(&path.to_string()) {
            Ok(temp_path) => GString::from(temp_path),
            Err(e) => {
                self.signals()
                    .log_sent()
                    .emit(&GString::from(format!("{e}")));

                return GString::from("");
            },
        };
    }

    #[signal]
    fn log_sent(message : GString);
}
