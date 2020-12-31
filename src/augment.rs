use ffmpeg::util::frame::video::Video;
use glium::implement_vertex;
use glium::{glutin, Surface};
use std::path::Path;
use std::sync::mpsc::*;
use std::thread;
use std::time::*;

mod shaders;
mod video;

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

    run_render(rx);
}

fn run_render(frame_rx: Receiver<Video>) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    implement_vertex!(Vertex, position);

    let vertex1 = Vertex {
        position: [-1.0, -1.0],
    };
    let vertex2 = Vertex {
        position: [-1.0, 1.0],
    };
    let vertex3 = Vertex {
        position: [1.0, 1.0],
    };
    let vertex4 = Vertex {
        position: [1.0, -1.0],
    };
    let shape = vec![vertex1, vertex2, vertex3, vertex4, vertex1, vertex3];
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let mut program_handle = shaders::ProgramHandle::load_and_watch(
        &display,
        Path::new("video.vert"),
        Path::new("video.frag"),
    )
    .unwrap();

    let now = Instant::now();

    let frame = frame_rx.recv().unwrap();
    let image = glium::texture::RawImage2d::from_raw_rgb(
        frame.data(0).to_vec(),
        (frame.width(), frame.height()),
    );
    let mut video_texture = glium::texture::Texture2d::new(&display, image).unwrap();

    event_loop.run(move |ev, _, control_flow| {
        program_handle.poll(&display);
        if let Ok(new_frame) = frame_rx.try_recv() {
            let image_dimensions = (new_frame.width(), new_frame.height());
            let image = glium::texture::RawImage2d::from_raw_rgb(
                new_frame.data(0).to_vec(),
                image_dimensions,
            );
            video_texture = glium::texture::Texture2d::new(&display, image).unwrap();
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;
        let resolution = [width as f32, height as f32, aspect_ratio];
        let program = program_handle.as_program();

        if program.is_ok() {
            let result = target.draw(
                &vertex_buffer,
                &indices,
                &(program).unwrap(),
                &uniform! {
                    iResolution: resolution,
                    iTime: now.elapsed().as_secs_f32(),
                    iVideo: &video_texture,
                },
                &Default::default(),
            );
            if result.is_err() {
                println!("Error drawing: {:?}", result.err());
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
                _ => return,
            },
            _ => (),
        }
    });
}
