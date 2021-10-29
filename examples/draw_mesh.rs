use oblivion::{GraphicsContext, Mesh, MeshBuilder, Render, Transform};
#[path = "common.rs"]
mod common;

struct DrawMeshExample {
    mesh: Mesh,
}

impl common::Example for DrawMeshExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let mesh = MeshBuilder::new()
            .quad([1.0, 1.0], [0.5, 0.5], [1.0, 0.0, 0.0])
            .quad([0.5, 0.5], [0.5, 0.5], [0.0, 1.0, 0.0])
            .quad([0.0, 0.0], [0.5, 0.5], [0.0, 0.0, 1.0])
            .build(&ctx);
        DrawMeshExample { mesh }
    }

    fn draw(&self, render: &mut Render) {
        oblivion::clear(render, [0.1, 0.2, 0.3, 1.0]);
        self.mesh.draw(render, Transform::default());
    }
}

fn main() {
    common::run::<DrawMeshExample>();
}
