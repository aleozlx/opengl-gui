extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate imgui;
extern crate imgui_gfx_renderer;
extern crate imgui_sys;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_window_glutin as gfx_glutin;
use imgui::*;

// mod support_gfx;

fn main() {
    support_gfx::run("hello_gfx.rs".to_owned(), CLEAR_COLOR, hello_world);
}
