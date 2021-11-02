use oblivion::{GraphicsContext, Mesh, MeshBuilder, Render, Transform};
#[path = "common.rs"]
mod common;

struct DrawMeshExample {
    mesh: Mesh,
}

impl common::Example for DrawMeshExample {
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
                oblivion::DrawMode::stroke(dimensions.x / 100.0),
            )
            .unwrap()
            .build(&ctx);
        DrawMeshExample { mesh }
    }

    fn draw(&self, render: &mut Render) {
        oblivion::clear(render, [0.1, 0.2, 0.3, 1.0]);
        self.mesh.draw(
            render,
            Transform {
                offset: [0.5, 0.5].into(),
                ..Default::default()
            },
        );
    }
}

fn main() {
    common::run::<DrawMeshExample>();
}
