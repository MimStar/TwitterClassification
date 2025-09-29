use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
struct CleanData {
    base: Base<Node>
}

#[godot_api]
impl INode for CleanData {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
        }
    }
}

#[godot_api]
impl CleanData {
    #[func]
    fn clean_data(&mut self, path : String) -> String {
        return path;
    }

    #[signal]
    fn log_sent(message : String);
}

fn rem_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str()
}