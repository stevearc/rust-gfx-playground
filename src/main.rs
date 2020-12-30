#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate notify;

mod render_teapot;
mod shadertoy;
mod teapot;

fn main() {
    // render_teapot::start();
    shadertoy::start();
}
