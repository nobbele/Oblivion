use std::time::Instant;

use image::GenericImageView;
use oblivion::{Angle, GraphicsContext, Image, Render, Transform};
#[path = "common.rs"]
mod common;

struct DrawImageExample {
    image: Image,
    start: Instant,
}

impl common::Example for DrawImageExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let image_bytes = include_bytes!("../resources/textures/happy-tree.png");
        let image_data = image::load_from_memory(image_bytes).unwrap();
        let image_rgba = image_data.as_rgba8().unwrap();
        let dimensions = image_data.dimensions();
        let image = Image::new(ctx, [dimensions.0, dimensions.1], image_rgba);
        DrawImageExample {
            image,
            start: Instant::now(),
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
        let elapsed = Angle::from_radians(self.start.elapsed().as_secs_f32() * 2.0);
        for i in 0..12 {
            self.image.draw(
                render,
                Transform {
                    scale: [
                        ((elapsed * 2.0).cos() * 0.5 + 1.0) / 4.0,
                        ((elapsed * 2.0).sin() * 0.5 + 1.0) / 4.0,
                    ]
                    .into(),
                    position: [
                        ((elapsed + i as f32 * std::f32::consts::FRAC_PI_6).sin() * 0.5 + 1.0)
                            / 2.0,
                        ((elapsed + i as f32 * std::f32::consts::FRAC_PI_6).cos() * 0.5 + 1.0)
                            / 2.0,
                    ]
                    .into(),
                    rotation: Angle::from_radians(elapsed.sin()),
                    ..Default::default()
                },
            );
        }
    }
}

fn main() {
    common::run::<DrawImageExample>();
}
