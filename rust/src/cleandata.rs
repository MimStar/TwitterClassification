use godot::prelude::*;

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
        self.signals().log_sent().emit(&path);
        return path;
    }

    #[signal]
    fn log_sent(message: GString);
}
