use image::GenericImageView;
use oblivion::{GraphicsContext, Image, Mesh, MeshBuilder, Render, Text, Transform};
#[path = "common.rs"]
mod common;

struct DrawProjectionExample {
    mesh: Mesh,
    text: Text,
    image: Image,
    dimensions: mint::Vector2<f32>,
}

impl common::Example for DrawProjectionExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let dimensions = ctx.surface_dimensions();
        let dimensions = mint::Vector2 {
            x: dimensions.x as f32,
            y: dimensions.y as f32,
        };
        ctx.set_projection(dimensions);
        let mesh = MeshBuilder::new()
            .circle(
                [dimensions.x / 2.0, dimensions.y / 2.0],
                [400.0, 400.0],
                [1.0, 0.0, 0.0, 1.0],
                0.01,
                oblivion::DrawMode::stroke(dimensions.x / 100.0),
            )
            .unwrap()
            .build(&ctx);
        let mut text = Text::new(ctx);
        text.add_text(["Projection Test!"]);
        text.flush(ctx);
        let image_bytes = include_bytes!("../resources/textures/happy-tree.png");
        let image_data = image::load_from_memory(image_bytes).unwrap();
        let image_rgba = image_data.as_rgba8().unwrap();
        let img_dimensions = image_data.dimensions();
        let image = Image::new(ctx, [img_dimensions.0, img_dimensions.1], image_rgba);
        DrawProjectionExample {
            mesh,
            text,
            image,
            dimensions,
        }
    }

    fn draw(&self, render: &mut Render) {
        oblivion::clear(render, [0.1, 0.2, 0.3, 1.0]);
        self.image.draw(
            render,
            Transform {
                position: [self.dimensions.x / 2.0, self.dimensions.y / 6.0].into(),
                scale: [0.25, 0.25].into(),
                ..Default::default()
            },
        );
        // Reminder, this mesh is created at the center so there's no need to position it.
        self.mesh.draw(render, Transform::default());
        self.text.draw(
            render,
            Transform {
                position: [self.dimensions.x / 2.0, self.dimensions.y / 6.0].into(),
                ..Default::default()
            },
        );
    }
}

fn main() {
    common::run::<DrawProjectionExample>();
}
