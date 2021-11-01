use oblivion::{GraphicsContext, Mesh, MeshBuilder, Render, Transform};
use rand::Rng;
#[path = "common.rs"]
mod common;

struct DrawMeshExample {
    mesh: Mesh,
}

impl common::Example for DrawMeshExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let mut rng = rand::thread_rng();
        let mut builder = MeshBuilder::new();
        for _ in 0..100 {
            builder
                .tri(
                    [rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)],
                    [rng.gen_range(0.25..=1.0), rng.gen_range(0.25..=1.0)],
                    [
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                        1.0,
                    ],
                    oblivion::DrawMode::fill(),
                )
                .unwrap()
                .quad(
                    [rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)],
                    [rng.gen_range(0.25..=1.0), rng.gen_range(0.25..=1.0)],
                    [
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                        1.0,
                    ],
                    oblivion::DrawMode::fill(),
                )
                .unwrap();
        }
        let mesh = builder.build(&ctx);
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
