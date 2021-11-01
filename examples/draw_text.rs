use oblivion::{GraphicsContext, Render, Text, Transform};
#[path = "common.rs"]
mod common;

struct DrawTextExample {
    text: Text,
    more_text: Text,
    count: u32,
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
    }
}

fn main() {
    common::run::<DrawTextExample>();
}
