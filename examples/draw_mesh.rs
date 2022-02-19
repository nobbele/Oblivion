use oblivion::{GraphicsContext, Mesh, MeshBuilder, Render, Transform};
#[path = "common.rs"]
mod common;

struct DrawMeshExample {
    mesh: Mesh,
}

fn create_mesh(ctx: &GraphicsContext) -> Mesh {
    MeshBuilder::new()
        .quad(
            [0.0, 0.0],
            [1.0 / 3.0, 1.0 / 3.0],
            [0.0, 0.0, 1.0, 1.0], // BLUE
            oblivion::DrawMode::fill(),
        )
        .unwrap()
        .quad(
            [1.0 / 3.0, 1.0 / 3.0],
            [1.0 / 3.0, 1.0 / 3.0],
            [0.0, 1.0, 0.0, 1.0], // GREEN
            oblivion::DrawMode::stroke(0.02),
        )
        .unwrap()
        .quad(
            [2.0 / 3.0, 2.0 / 3.0],
            [1.0 / 3.0, 1.0 / 3.0],
            [1.0, 0.0, 0.0, 1.0], // RED
            oblivion::DrawMode::fill(),
        )
        .unwrap()
        .tri(
            [0.0, 2.0 / 3.0],
            [1.0 / 3.0, 1.0 / 3.0],
            [1.0, 1.0, 0.0, 1.0], // YELLOW
            oblivion::DrawMode::stroke(0.02),
        )
        .unwrap()
        .circle(
            [2.0 / 3.0, 0.0],
            [1.0 / 3.0, 1.0 / 3.0],
            [1.0, 1.0, 1.0, 1.0], // WHITE
            0.01,
            oblivion::DrawMode::stroke(0.02),
        )
        .unwrap()
        .build(&ctx)
}

impl common::Example for DrawMeshExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let mesh = create_mesh(ctx);
        DrawMeshExample { mesh }
    }

    fn update(&mut self, ctx: &mut GraphicsContext) {
        self.mesh = create_mesh(ctx);
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
