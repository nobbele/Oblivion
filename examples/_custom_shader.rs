use oblivion::{GraphicsContext, Mesh, MeshBuilder, Render};
use wgpu::include_wgsl;
#[path = "common.rs"]
mod common;

// TODO

struct DrawImageExample {
    mesh: Mesh,
}

impl common::Example for DrawImageExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let _source = include_wgsl!("custom.wgsl");
        let mesh = MeshBuilder::new()
            .quad([0.0, 0.0], [0.5, 0.5], [0.0, 1.0, 0.0])
            .build(&ctx);
        DrawImageExample { mesh }
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
    common::run::<DrawImageExample>();
}
