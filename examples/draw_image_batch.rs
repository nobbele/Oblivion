use image::GenericImageView;
use oblivion::{GraphicsContext, ImageBatch, Render, Transform};
#[path = "common.rs"]
mod common;

struct DrawImageBatchExample {
    batch: ImageBatch,
}

impl common::Example for DrawImageBatchExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let image_bytes = include_bytes!("../resources/textures/happy-tree.png");
        let image_data = image::load_from_memory(image_bytes).unwrap();
        let image_rgba = image_data.as_rgba8().unwrap();
        let dimensions = image_data.dimensions();
        let mut batch = ImageBatch::new(ctx, dimensions.0, dimensions.1, image_rgba);
        batch.add_instance(
            ctx,
            &[
                Transform {
                    position: [0.25, 0.25],
                    scale: [0.25, 0.5],
                    ..Default::default()
                },
                Transform {
                    position: [0.75, 0.75],
                    scale: [0.5, 0.25],
                    ..Default::default()
                },
            ],
        );
        let transforms = (0..1)
            .map(|_| Transform {
                position: [0.5, 0.5],
                scale: [0.25, 0.25],
                ..Default::default()
            })
            .collect::<Vec<_>>();
        batch.add_instance(ctx, &transforms);
        DrawImageBatchExample { batch }
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
        self.batch.draw(
            render,
            Transform {
                position: [0.0, 0.0],
                scale: [1.0, 1.0],
                ..Default::default()
            },
        );
    }
}

fn main() {
    common::run::<DrawImageBatchExample>();
}
