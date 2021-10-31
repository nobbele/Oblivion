use oblivion::{GraphicsContext, Render, Text, Transform};
#[path = "common.rs"]
mod common;

struct DrawTextExample {
    text: Text,
    count: u32,
}

impl common::Example for DrawTextExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        DrawTextExample {
            text: Text::new(ctx),
            count: 0,
        }
    }

    fn update(&mut self, ctx: &mut GraphicsContext) {
        self.count += 1;
        self.text.clear();
        self.text.add_text([format!("Frame Count: {}", self.count)]);
        self.text.flush(ctx);
    }

    fn draw(&self, render: &mut Render) {
        oblivion::clear(render, [0.1, 0.2, 0.3, 1.0]);
        self.text.draw(
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
