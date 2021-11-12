#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate ffmpeg_next as ffmpeg;
extern crate notify;

mod augment;
mod particles;
mod render_teapot;
mod shadertoy;
mod teapot;

fn main() {
    // render_teapot::start();
    // shadertoy::start();
    augment::start();
}
