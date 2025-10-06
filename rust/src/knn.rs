use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
struct Knn {
    base: Base<Node>,
}

#[godot_api]
impl INode for Knn {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl Knn {
    #[func]
    fn knn_execute(&mut self, path: GString) -> GString {
        
    }

    #[signal]
    fn log_sent(message : GString);
}