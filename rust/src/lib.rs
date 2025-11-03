use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

mod cleandata;
mod knn;
mod regex_ext;
mod csv_ext;
mod naive_classification;