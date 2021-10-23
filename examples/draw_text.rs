use oblivion::{GraphicsContext, Render, Text, Transform};
#[path = "common.rs"]
mod common;

struct DrawTextExample {
    text: Text,
}

impl common::Example for DrawTextExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let text = Text::new(ctx);
        DrawTextExample { text }
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
        self.text.draw(render, Transform::default());
    }
}

fn main() {
    common::run::<DrawTextExample>();
}
