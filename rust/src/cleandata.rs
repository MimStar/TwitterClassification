use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
struct CleanData {
    base: Base<Node>
}

#[godot_api]
impl INode for CleanData {
    fn init(base: Base<Node>) -> Self {
        godot_print!("Clean Data rust node loaded!"); // Prints to the Godot console
        
        Self {
            base,
        }
    }
}

#[godot_api]
impl CleanData {
    #[func]
    fn clean_data() {
        godot_print!("Test");
    }
}