use ffmpeg::util::frame::video::Video;
use ffmpeg::{
    format::Pixel,
    software::scaling::{Context, Flags},
};
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

use self::filters::ConnectedComponent;
#[allow(unused_imports)]
use self::filters::{bgsub, blur, denoise, edges, find_objects, pixelate};

mod filters;
pub mod shaders;
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
        let filename = Path::new("Bliss Dance - Nicky Evers.mp4");
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

    let mut program_handle = shaders::ProgramHandle::new(
        &display,
        Path::new("shaders/video.vert"),
        Path::new("shaders/video.frag"),
    )
    .unwrap();
    let mut obj_prog_handle = shaders::ProgramHandle::new(
        &display,
        Path::new("shaders/obj.vert"),
        Path::new("shaders/obj.frag"),
    )
    .unwrap();

    let frame = rx.recv().unwrap();
    let mut processor = ImageProcessor::new(frame.width(), frame.height()).unwrap();
    let mut split_screen = true;
    let mut fullscreen = false;
    // TODO
    // let p = ParticleSystem::new().
    // let particles = vec![ParticleSystemRunner::new(display, ];

    let mut last_frame = Instant::now();
    event_loop.run(move |ev, _, control_flow| {
        let frame_start = now.elapsed().as_micros();
        let delta = last_frame.elapsed().as_secs_f32();
        last_frame = Instant::now();
        if cfg!(debug_assertions) {
            program_handle.poll(&display);
            obj_prog_handle.poll(&display);
        }
        if let Ok(new_frame) = rx.try_recv() {
            let image = glium::texture::RawImage2d::from_raw_rgb(
                new_frame.data(0).to_vec(),
                (new_frame.width(), new_frame.height()),
            );
            let video_texture = glium::texture::Texture2d::new(&display, image).unwrap();

            let (components_frame, components) = processor
                .find_components_with_intermediate_frame(&display, &new_frame)
                .unwrap();

            let mut objects = vec![];
            for component in components {
                let left = component.left as f32 / new_frame.width() as f32;
                let right =
                    (component.left as f32 + component.width as f32) / new_frame.width() as f32;
                let top = component.top as f32 / new_frame.height() as f32;
                let bottom =
                    (component.top as f32 + component.height as f32) / new_frame.height() as f32;
                // The video is upside down because it goes from top to bottom and GL is from
                // bottom to top
                let top = 1f32 - top;
                let bottom = 1f32 - bottom;
                // Shift this into the lower left panel
                let top = top - 1f32;
                let bottom = bottom - 1f32;
                let left = left - 1f32;
                let right = right - 1f32;
                objects.push(Panel {
                    vbo: make_square([left, top], [right, bottom], &display).unwrap(),
                    indices: glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                });
            }

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
                            iVideo: &video_texture,
                        },
                    );

                    panel_upper_right.draw(
                        &mut target,
                        prog,
                        &uniform! {
                            iResolution: resolution,
                            iTime: now.elapsed().as_secs_f32(),
                            iVideo: &components_frame,
                        },
                    );

                    panel_lower_left.draw(
                        &mut target,
                        prog,
                        &uniform! {
                            iResolution: resolution,
                            iTime: now.elapsed().as_secs_f32(),
                            iVideo: &video_texture,
                        },
                    );

                    panel_lower_right.draw(
                        &mut target,
                        prog,
                        &uniform! {
                            iResolution: resolution,
                            iTime: now.elapsed().as_secs_f32(),
                            iVideo: &video_texture,
                        },
                    );

                    for obj in objects {
                        obj.draw(
                            &mut target,
                            obj_prog_handle.as_program().unwrap(),
                            &uniform! {
                                iResolution: resolution,
                                iTime: now.elapsed().as_secs_f32(),
                            },
                        );
                    }
                } else {
                    main_panel.draw(
                        &mut target,
                        prog,
                        &uniform! {
                            iResolution: resolution,
                            iTime: now.elapsed().as_secs_f32(),
                            iVideo: &video_texture,
                        },
                    );
                }
            }

            target.finish().unwrap();
        }

        let work = now.elapsed().as_micros() - frame_start;
        let sleep = std::cmp::max(1, 16_666_667 - work);
        let next_frame_time = Instant::now() + Duration::from_nanos(sleep as u64);
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

struct ImageProcessor {
    rgb2bgr_ctx: Context,
    bgr2rgb_ctx: Context,
}

impl ImageProcessor {
    fn new(width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let rgb2bgr_ctx = Context::get(
            Pixel::RGB24,
            width,
            height,
            Pixel::BGR24,
            width,
            height,
            Flags::BILINEAR,
        )?;
        let bgr2rgb_ctx = Context::get(
            Pixel::BGR24,
            width,
            height,
            Pixel::RGB24,
            width,
            height,
            Flags::BILINEAR,
        )?;
        Ok(ImageProcessor {
            rgb2bgr_ctx,
            bgr2rgb_ctx,
        })
    }

    fn find_components_with_intermediate_frame(
        &mut self,
        display: &glium::Display,
        frame: &ffmpeg::frame::Video,
    ) -> Result<
        (glium::texture::Texture2d, Vec<ConnectedComponent>),
        Box<dyn std::error::Error + 'static>,
    > {
        let mut bgr_frame = Video::empty();
        self.rgb2bgr_ctx.run(&frame, &mut bgr_frame)?;

        let mut components_frame =
            Video::new(bgr_frame.format(), bgr_frame.width(), bgr_frame.height());
        let components = find_objects(&bgr_frame, Some(&mut components_frame))?;

        let mut final_frame = Video::empty();
        self.bgr2rgb_ctx.run(&components_frame, &mut final_frame)?;
        let image = glium::texture::RawImage2d::from_raw_rgb(
            final_frame.data(0).to_vec(),
            (frame.width(), frame.height()),
        );
        Ok((glium::texture::Texture2d::new(display, image)?, components))
    }
}
