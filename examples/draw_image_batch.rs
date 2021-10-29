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
        let mut batch = ImageBatch::new(ctx, [dimensions.0, dimensions.1], image_rgba);
        batch.add_instance(
            ctx,
            &[
                Transform {
                    position: [0.25, 0.25].into(),
                    scale: [0.25, 0.5].into(),
                    ..Default::default()
                },
                Transform {
                    position: [0.75, 0.75].into(),
                    scale: [0.5, 0.25].into(),
                    ..Default::default()
                },
            ],
        );
        let transforms = (0..1)
            .map(|_| Transform {
                position: [0.5, 0.5].into(),
                scale: [0.25, 0.25].into(),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        batch.add_instance(ctx, &transforms);
        DrawImageBatchExample { batch }
    }

    fn draw(&self, render: &mut Render) {
        oblivion::clear(render, [0.1, 0.2, 0.3, 1.0]);
        self.batch.draw(
            render,
            Transform {
                position: [0.0, 0.0].into(),
                scale: [1.0, 1.0].into(),
                ..Default::default()
            },
        );
    }
}

fn main() {
    common::run::<DrawImageBatchExample>();
}
