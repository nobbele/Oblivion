use oblivion::{GraphicsContext, Render, Text, Transform};
#[path = "common.rs"]
mod common;

struct DrawTextExample {
    text: Text,
    more_text: Text,
    count: u32,
    mesh: oblivion::Mesh,
}

impl common::Example for DrawTextExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let mut more_text = Text::new(ctx);
        more_text.add_text(["Hello ", "World\n"]);
        more_text.add_text(["Foobar"]);
        more_text.flush(ctx);
        DrawTextExample {
            text: Text::new(ctx),
            more_text,
            count: 0,
            mesh: oblivion::Mesh::new(ctx, &[], &[]),
        }
    }

    fn update(&mut self, ctx: &mut GraphicsContext) {
        self.count += 1;
        self.text.clear();
        self.text.add_text([format!("Frame Count: {}", self.count)]);
        self.text.flush(ctx);

        let text_dim = self.text.bounds();
        let mesh = oblivion::MeshBuilder::new()
            .quad(
                text_dim.0,
                text_dim.1,
                rgb::RGBA {
                    r: 1.0,
                    g: 0.0,
                    b: 1.0,
                    a: 0.5,
                },
                oblivion::DrawMode::fill(),
            )
            .unwrap()
            .build(ctx);
        self.mesh = mesh;
    }

    fn draw(&self, render: &mut Render) {
        oblivion::clear(render, [0.1, 0.2, 0.3, 1.0]);
        self.more_text.draw(
            render,
            Transform {
                offset: [0.0, 0.0].into(),
                position: [0.0, 0.0].into(),
                scale: [0.5, 0.5].into(),
                ..Default::default()
            },
        );
        self.text.draw(
            render,
            Transform {
                position: [0.5, 0.5].into(),
                scale: [0.5, 0.5].into(),
                ..Default::default()
            },
        );
        self.mesh.draw(
            render,
            Transform {
                position: [0.5, 0.5].into(),
                scale: [0.5, 0.5].into(),
                ..Default::default()
            },
        );
    }
}

fn main() {
    common::run::<DrawTextExample>();
}
