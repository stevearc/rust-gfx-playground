use glium::implement_vertex;
use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::*;

pub fn start() {
    use glium::{glutin, Surface};

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

    let mut program = load_shader(&display);

    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_millis(50)).unwrap();
    watcher
        .watch("shader.vert", RecursiveMode::NonRecursive)
        .unwrap();
    watcher
        .watch("shader.frag", RecursiveMode::NonRecursive)
        .unwrap();
    let now = Instant::now();

    event_loop.run(move |ev, _, control_flow| {
        if rx.try_recv().is_ok() {
            program = load_shader(&display);
            if program.is_err() {
                println!("Error loading shader: {:?}", program.as_ref().err());
            }
            watcher
                .watch("shader.vert", RecursiveMode::NonRecursive)
                .unwrap();
            watcher
                .watch("shader.frag", RecursiveMode::NonRecursive)
                .unwrap();
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;
        let resolution = [width as f32, height as f32, aspect_ratio];
        if program.is_ok() {
            let result = target.draw(
                &vertex_buffer,
                &indices,
                &program.as_ref().unwrap(),
                &uniform! {
                    iResolution: resolution,
                    iTime: now.elapsed().as_secs_f32(),
                },
                &Default::default(),
            );
            if result.is_err() {
                println!("Error drawing: {:?}", result.err());
            }
        }

        target.finish().unwrap();

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

fn load_shader(display: &glium::Display) -> Result<glium::Program, Box<dyn std::error::Error>> {
    let vert = std::fs::read_to_string("shader.vert")?;
    let frag = std::fs::read_to_string("shader.frag")?;
    Ok(glium::Program::from_source(display, &vert, &frag, None)?)
}
