pub mod camera;
pub mod drawable;
pub mod gpu_immediate;
pub mod mesh;
pub mod meshreader;
pub mod shader;
pub mod util;

// expose other crates as public for easier usage.
pub use egui;
pub use egui_glfw;
pub use gl;
pub use glfw;
pub use nalgebra_glm as glm;

extern crate generational_arena;
extern crate itertools;
