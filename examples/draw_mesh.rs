use oblivion::{GraphicsContext, Mesh, MeshBuilder, Render, Transform};
#[path = "common.rs"]
mod common;

struct DrawMeshExample {
    mesh: Mesh,
}

impl common::Example for DrawMeshExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let mesh = MeshBuilder::new()
            .quad(
                [0.0, 0.0],
                [1.0 / 3.0, 1.0 / 3.0],
                [0.0, 0.0, 1.0],
                oblivion::DrawMode::fill(),
            )
            .quad(
                [1.0 / 3.0, 1.0 / 3.0],
                [1.0 / 3.0, 1.0 / 3.0],
                [0.0, 1.0, 0.0],
                oblivion::DrawMode::stroke(0.02),
            )
            .quad(
                [2.0 / 3.0, 2.0 / 3.0],
                [1.0 / 3.0, 1.0 / 3.0],
                [1.0, 0.0, 0.0],
                oblivion::DrawMode::fill(),
            )
            .tri(
                [0.0, 2.0 / 3.0],
                [1.0 / 3.0, 1.0 / 3.0],
                [1.0, 0.0, 0.0],
                oblivion::DrawMode::stroke(0.02),
            )
            .circle(
                [2.0 / 3.0, 0.0],
                [1.0 / 3.0, 1.0 / 3.0],
                [1.0, 0.0, 0.0],
                oblivion::DrawMode::stroke(0.02),
            )
            .build(&ctx);
        DrawMeshExample { mesh }
    }

    fn draw(&self, render: &mut Render) {
        oblivion::clear(render, [0.1, 0.2, 0.3, 1.0]);
        self.mesh.draw(
            render,
            Transform {
                offset: [0.0, 0.0].into(),
                ..Default::default()
            },
        );
    }
}

fn main() {
    common::run::<DrawMeshExample>();
}
