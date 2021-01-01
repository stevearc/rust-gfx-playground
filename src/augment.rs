use glium::{
    glutin::event::{ElementState, VirtualKeyCode},
    implement_vertex,
};
use glium::{
    glutin::{self, window::Fullscreen},
    Surface,
};
use std::path::Path;
use std::sync::mpsc::*;
use std::thread;
use std::time::*;

mod shaders;
mod video;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

#[allow(dead_code)]
pub fn start() {
    let (tx, rx) = channel();

    thread::spawn(move || {
        let filename = Path::new("POISONS.mp4");
        let result = video::load_video(&filename, tx);
        if result.is_err() {
            println!("Error loading video: {:?}", result.err().unwrap());
        }
    });

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    implement_vertex!(Vertex, position, tex_coords);

    let now = Instant::now();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let main_panel = Panel {
        vbo: make_square([-1.0, -1.0], [1.0, 1.0], &display).unwrap(),
        indices,
    };
    let panel_upper_left = Panel {
        vbo: make_square([-1.0, 0.0], [0.0, 1.0], &display).unwrap(),
        indices,
    };
    let panel_upper_right = Panel {
        vbo: make_square([0.0, 0.0], [1.0, 1.0], &display).unwrap(),
        indices,
    };
    let panel_lower_left = Panel {
        vbo: make_square([-1.0, -1.0], [0.0, 0.0], &display).unwrap(),
        indices,
    };
    let panel_lower_right = Panel {
        vbo: make_square([0.0, -1.0], [1.0, 0.0], &display).unwrap(),
        indices,
    };

    let mut program_handle = shaders::ProgramHandle::load_and_watch(
        &display,
        Path::new("video.vert"),
        Path::new("video.frag"),
    )
    .unwrap();

    let frame = rx.recv().unwrap();
    let mut image_cache = CachedGLValue::new(
        &display,
        &frame,
        Box::new(|display: &glium::Display, frame: &ffmpeg::frame::Video| {
            let image = glium::texture::RawImage2d::from_raw_rgb(
                frame.data(0).to_vec(),
                (frame.width(), frame.height()),
            );
            glium::texture::Texture2d::new(display, image).unwrap()
        }),
    );
    let mut split_screen = false;
    let mut fullscreen = false;

    event_loop.run(move |ev, _, control_flow| {
        program_handle.poll(&display);
        if let Ok(new_frame) = rx.try_recv() {
            image_cache.update(&display, &new_frame);
        }
        let video_texture = &image_cache.cached;

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;
        let resolution = [width as f32, height as f32, aspect_ratio];
        let program = program_handle.as_program();
        if program.is_ok() {
            let prog = program.unwrap();

            if split_screen {
                panel_upper_left.draw(
                    &mut target,
                    prog,
                    &uniform! {
                        iResolution: resolution,
                        iTime: now.elapsed().as_secs_f32(),
                        iVideo: video_texture,
                    },
                );

                panel_upper_right.draw(
                    &mut target,
                    prog,
                    &uniform! {
                        iResolution: resolution,
                        iTime: now.elapsed().as_secs_f32(),
                        iVideo: video_texture,
                    },
                );

                panel_lower_left.draw(
                    &mut target,
                    prog,
                    &uniform! {
                        iResolution: resolution,
                        iTime: now.elapsed().as_secs_f32(),
                        iVideo: video_texture,
                    },
                );

                panel_lower_right.draw(
                    &mut target,
                    prog,
                    &uniform! {
                        iResolution: resolution,
                        iTime: now.elapsed().as_secs_f32(),
                        iVideo: video_texture,
                    },
                );
            } else {
                main_panel.draw(
                    &mut target,
                    prog,
                    &uniform! {
                        iResolution: resolution,
                        iTime: now.elapsed().as_secs_f32(),
                        iVideo: video_texture,
                    },
                );
            }
        }

        target.finish().unwrap();

        // TODO: shouldn't this be variable based on how much work we've done?
        let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                    if let ElementState::Pressed = input.state {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::F) => {
                                if fullscreen {
                                    display.gl_window().window().set_fullscreen(None);
                                } else {
                                    let monitor_handle = display
                                        .gl_window()
                                        .window()
                                        .available_monitors()
                                        .next()
                                        .unwrap();
                                    let fs = Fullscreen::Borderless(Some(monitor_handle));
                                    display.gl_window().window().set_fullscreen(Some(fs));
                                }
                                fullscreen = !fullscreen;
                            }
                            Some(VirtualKeyCode::S) => {
                                split_screen = !split_screen;
                            }
                            _ => {}
                        }
                    }
                }
                _ => return,
            },
            _ => (),
        }
    });
}

fn make_square(
    upper_left: [f32; 2],
    lower_right: [f32; 2],
    display: &glium::Display,
) -> Result<glium::VertexBuffer<Vertex>, glium::vertex::BufferCreationError> {
    let vertex1 = Vertex {
        position: upper_left,
        tex_coords: [0.0, 0.0],
    };
    let vertex2 = Vertex {
        position: [upper_left[0], lower_right[1]],
        tex_coords: [0.0, 1.0],
    };
    let vertex3 = Vertex {
        position: lower_right,
        tex_coords: [1.0, 1.0],
    };
    let vertex4 = Vertex {
        position: [lower_right[0], upper_left[1]],
        tex_coords: [1.0, 0.0],
    };
    let shape = vec![vertex1, vertex2, vertex3, vertex4, vertex1, vertex3];
    glium::VertexBuffer::new(display, &shape)
}

struct Panel {
    vbo: glium::VertexBuffer<Vertex>,
    indices: glium::index::NoIndices,
}

impl Panel {
    fn draw<U>(&self, target: &mut glium::Frame, program: &glium::Program, uniforms: &U)
    where
        U: glium::uniforms::Uniforms,
    {
        let result = target.draw(
            &self.vbo,
            &self.indices,
            program,
            uniforms,
            &Default::default(),
        );
        if result.is_err() {
            println!("Error drawing: {:?}", result.err());
        }
    }
}

struct CachedGLValue<T, U> {
    callback: Box<dyn Fn(&glium::Display, &T) -> U>,
    cached: U,
}

impl<T, U> CachedGLValue<T, U> {
    fn new(
        display: &glium::Display,
        value: &T,
        callback: Box<dyn Fn(&glium::Display, &T) -> U>,
    ) -> CachedGLValue<T, U> {
        let cached = callback(display, value);
        return CachedGLValue { callback, cached };
    }

    fn update(&mut self, display: &glium::Display, new_value: &T) {
        self.cached = (*self.callback)(display, new_value);
    }
}
