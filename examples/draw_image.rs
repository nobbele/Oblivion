use image::GenericImageView;
use oblivion::{GraphicsContext, Image, Render, Transform};
#[path = "common.rs"]
mod common;

struct DrawImageExample {
    image: Image,
}

impl common::Example for DrawImageExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let image_bytes = include_bytes!("../resources/textures/happy-tree.png");
        let image_data = image::load_from_memory(image_bytes).unwrap();
        let image_rgba = image_data.as_rgba8().unwrap();
        let dimensions = image_data.dimensions();
        let image = Image::new(ctx, [dimensions.0, dimensions.1], image_rgba);
        DrawImageExample { image }
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
        for _ in 0..1 {
            self.image.draw(
                render,
                Transform {
                    position: [0.5, 0.5].into(),
                    scale: [0.25, 0.25].into(),
                    ..Default::default()
                },
            );
        }
    }
}

fn main() {
    common::run::<DrawImageExample>();
}
