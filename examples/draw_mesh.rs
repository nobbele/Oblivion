use oblivion::{GraphicsContext, Mesh, MeshBuilder, Render};
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
            builder = builder
                .tri(
                    [rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)],
                    [rng.gen_range(0.25..=1.0), rng.gen_range(0.25..=1.0)],
                    [
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                    ],
                )
                .quad(
                    [rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)],
                    [rng.gen_range(0.25..=1.0), rng.gen_range(0.25..=1.0)],
                    [
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                    ],
                )
        }
        let mesh = builder.build(&ctx);
        DrawMeshExample { mesh }
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
        self.mesh.draw(render);
    }
}

fn main() {
    common::run::<DrawMeshExample>();
}
