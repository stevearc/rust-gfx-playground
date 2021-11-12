use std::{
    error::Error,
    path::Path,
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

pub struct ProgramHandle<'a> {
    vertex_shader: &'a Path,
    fragment_shader: &'a Path,
    watcher: RecommendedWatcher,
    listener: Receiver<DebouncedEvent>,
    program: Result<glium::Program, Box<dyn Error>>,
}

impl<'a> std::fmt::Debug for ProgramHandle<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ProgramHandle({:?}, {:?})",
            self.vertex_shader, self.fragment_shader
        )
    }
}

impl<'a> ProgramHandle<'a> {
    #[cfg(debug_assertions)]
    pub fn poll(&mut self, display: &glium::Display) {
        if self.listener.try_recv().is_ok() {
            let new_prog = load_program(display, &*self.vertex_shader, &*self.fragment_shader);
            self.watcher
                .watch(self.vertex_shader, RecursiveMode::NonRecursive)
                .unwrap();
            self.watcher
                .watch(self.fragment_shader, RecursiveMode::NonRecursive)
                .unwrap();
            self.program = new_prog;
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn poll(&mut self, display: &glium::Display) {}

    pub fn as_program(&self) -> Result<&glium::Program, &Box<dyn Error>> {
        let progref = self.program.as_ref();
        let prog = progref?;
        Ok(&prog)
    }

    pub fn new(
        display: &glium::Display,
        vertex_shader: &'a Path,
        fragment_shader: &'a Path,
    ) -> Result<ProgramHandle<'a>, Box<dyn Error>> {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_millis(50)).unwrap();
        if cfg!(debug_assertions) {
            watcher
                .watch(&vertex_shader, RecursiveMode::NonRecursive)
                .unwrap();
            watcher
                .watch(&fragment_shader, RecursiveMode::NonRecursive)
                .unwrap();
        }

        let program = load_program(&display, &vertex_shader, &fragment_shader);
        Ok(ProgramHandle {
            program,
            watcher,
            listener: rx,
            vertex_shader,
            fragment_shader,
        })
    }
}

pub fn load_program(
    display: &glium::Display,
    vertex_shader: &Path,
    fragment_shader: &Path,
) -> Result<glium::Program, Box<dyn std::error::Error>> {
    let vert = std::fs::read_to_string(&vertex_shader)?;
    let frag = std::fs::read_to_string(&fragment_shader)?;
    Ok(glium::Program::from_source(display, &vert, &frag, None)?)
}
