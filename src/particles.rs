use std::path::Path;

use crate::augment::shaders::ProgramHandle;
use cgmath::{prelude::*, Vector2, Vector3};
use glium::{Display, Frame};

#[derive(Copy, Clone)]
struct Particle {
    position: Vector2<f32>,
    velocity: Vector2<f32>,
    life: u32,
}

impl Particle {
    fn is_alive(&self) -> bool {
        return self.life > 0;
    }
}

impl Particle {
    fn new() -> Particle {
        Particle {
            position: Vector2::new(0., 0.),
            velocity: Vector2::new(0., 0.),
            life: 0,
        }
    }
}

struct ParticleSystem {
    position: Vector2<f32>,
    num_particles: usize,
    lifetime: f32,
    emission_rate: f32,
    // texture
    // gravity
    // size_curve
    // color_curve
    // opacity_curve
}

impl ParticleSystem {
    pub fn new() -> ParticleSystem {
        return ParticleSystem {
            position: Vector2::new(0., 0.),
            num_particles: 10,
            lifetime: 5.,
            emission_rate: 2.,
        };
    }

    pub fn set_position(&mut self, position: Vector2<f32>) -> &mut ParticleSystem {
        self.position = position;
        self
    }
    pub fn set_num_particles(&mut self, num_particles: usize) -> &mut ParticleSystem {
        self.num_particles = num_particles;
        self
    }
    pub fn set_lifetime(&mut self, lifetime: f32) -> &mut ParticleSystem {
        self.lifetime = lifetime;
        self
    }
    pub fn set_emission_rate(&mut self, emission_rate: f32) -> &mut ParticleSystem {
        self.emission_rate = emission_rate;
        self
    }
}

struct ParticleSystemRunner<'a> {
    system: &'a ParticleSystem,
    particles: Vec<Particle>,
    program: ProgramHandle<'a>,
}

impl<'a> ParticleSystemRunner<'a> {
    pub fn new(display: &Display, system: &'a ParticleSystem) -> ParticleSystemRunner<'a> {
        return ParticleSystemRunner {
            system,
            particles: vec![Particle::new(); system.num_particles],
            program: ProgramHandle::new(
                display,
                Path::new("shaders/particle.vert"),
                Path::new("shaders/particle.frag"),
            )
            .unwrap(),
        };
    }

    pub fn run(&mut self, display: &Display, target: &mut Frame) {
        if cfg!(debug_assertions) {
            self.program.poll(&display);
        }
    }
}
