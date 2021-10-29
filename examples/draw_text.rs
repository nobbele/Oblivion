use oblivion::{GraphicsContext, Render, Text, Transform};
#[path = "common.rs"]
mod common;

struct DrawTextExample {
    text: Text,
}

impl common::Example for DrawTextExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let mut text = Text::new(ctx);
        text.add_text(ctx, &["Hello World"]).unwrap();
        DrawTextExample { text }
    }

    fn draw(&self, render: &mut Render) {
        oblivion::clear(render, [0.1, 0.2, 0.3, 1.0]);
        self.text.draw(
            render,
            Transform {
                position: [0.5, 0.5].into(),
                ..Default::default()
            },
        );
    }
}

fn main() {
    common::run::<DrawTextExample>();
}
