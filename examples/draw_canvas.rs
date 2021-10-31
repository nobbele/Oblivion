use oblivion::{Canvas, GraphicsContext, Render, Text, Transform};
#[path = "common.rs"]
mod common;

struct DrawCanvasExample {
    canvas_hidden: Canvas,
    canvas_shown: Canvas,
    text_hidden: Text,
    text_shown: Text,
    text_real: Text,
}

impl common::Example for DrawCanvasExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let canvas_hidden = Canvas::new(ctx, [64, 64]);
        let canvas_shown = Canvas::new(ctx, [64, 64]);
        let mut text_hidden = Text::new(ctx);
        text_hidden.add_text(["Hidden"]);
        let mut text_shown = Text::new(ctx);
        text_shown.add_text(["Shown"]);
        let mut text_real = Text::new(ctx);
        text_real.add_text(["Real"]);
        DrawCanvasExample {
            canvas_hidden,
            canvas_shown,
            text_hidden,
            text_shown,
            text_real,
        }
    }

    fn draw(&self, render: &mut Render) {
        oblivion::clear(render, [0.1, 0.2, 0.3, 1.0]);
        oblivion::push_canvas(render, &self.canvas_hidden);
        self.text_hidden.draw(
            render,
            Transform {
                position: [0.5, 0.5].into(),
                ..Default::default()
            },
        );
        oblivion::pop_canvas(render);

        oblivion::push_canvas(render, &self.canvas_shown);
        oblivion::clear(render, [1.0, 0.2, 0.3, 1.0]);
        self.text_shown.draw(
            render,
            Transform {
                position: [0.5, 0.5].into(),
                ..Default::default()
            },
        );
        oblivion::pop_canvas(render);
        self.canvas_shown.draw(
            render,
            Transform {
                position: [0.33, 0.5].into(),
                scale: [0.25, 0.25].into(),
                ..Default::default()
            },
        );
        self.canvas_shown.draw(
            render,
            Transform {
                position: [0.66, 0.5].into(),
                scale: [0.25, 0.25].into(),
                ..Default::default()
            },
        );
        self.text_real.draw(
            render,
            Transform {
                position: [0.5, 0.25].into(),
                scale: [0.25, 0.25].into(),
                ..Default::default()
            },
        );
    }
}

fn main() {
    common::run::<DrawCanvasExample>();
}
