use std::time::Instant;

use image::GenericImageView;
use oblivion::{GraphicsContext, Image, Render, Transform};
#[path = "common.rs"]
mod common;

struct Bunny {
    position: [f32; 2],
    velocity: [f32; 2],
}

struct DrawImageExample {
    image: Image,
    bunnies: Vec<Bunny>,
    prev: Instant,
    prev2: Instant,
}

impl common::Example for DrawImageExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let image_bytes = include_bytes!("../resources/textures/bunny.png");
        let image_data = image::load_from_memory(image_bytes).unwrap();
        let image_rgba = image_data.as_rgba8().unwrap();
        let dimensions = image_data.dimensions();
        let image = Image::new(ctx, dimensions.0, dimensions.1, image_rgba);
        DrawImageExample {
            image,
            bunnies: Vec::new(),
            prev: Instant::now(),
            prev2: Instant::now(),
        }
    }

    fn update(&mut self, _ctx: &mut GraphicsContext) {
        if self.prev.elapsed().as_secs_f32() > 0.1 {
            // Increase by 2%
            for i in 0..((self.bunnies.len() as f32 * 0.02) as u32).max(50) {
                let n = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .subsec_millis()
                    .wrapping_mul(self.bunnies.len() as u32 + i) as f32;
                self.bunnies.push(Bunny {
                    position: [0.5, 0.5],
                    velocity: [n.cos() * 0.01, n.sin() * 0.01],
                });
            }
            self.prev = Instant::now();
        }

        if self.prev2.elapsed().as_secs_f32() > 1.0 {
            println!("Bunny Count: {}", self.bunnies.len());
            self.prev2 = Instant::now();
        }

        for bunny in &mut self.bunnies {
            bunny.position[0] += bunny.velocity[0];
            bunny.position[1] += bunny.velocity[1];
            if bunny.velocity[1].abs() < 0.02 {
                bunny.velocity[1] -= 0.01; // Gravity
            }

            if (bunny.position[0] <= 0.0 && bunny.velocity[0] < 0.0)
                || (bunny.position[0] >= 1.0 && bunny.velocity[0] > 0.0)
            {
                bunny.velocity[0] *= -1.0;
            }

            if (bunny.position[1] <= 0.0 && bunny.velocity[1] < 0.0)
                || (bunny.position[1] >= 1.0 && bunny.velocity[1] > 0.0)
            {
                bunny.velocity[1] *= -1.0;
            }
        }
    }

    fn draw(&self, render: &mut Render) {
        oblivion::clear(
            render,
            wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        );
        for bunny in &self.bunnies {
            self.image.draw(
                render,
                Transform {
                    position: bunny.position,
                    scale: [1.0 / 12.0, 1.0 / 12.0],
                    ..Default::default()
                },
            );
        }
    }
}

fn main() {
    common::run::<DrawImageExample>();
}
