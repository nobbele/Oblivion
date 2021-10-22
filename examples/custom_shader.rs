use image::GenericImageView;
use oblivion::{GraphicsContext, Image, Render, Shader, Transform};
#[path = "common.rs"]
mod common;

// TODO

struct DrawImageExample {
    shader: Shader,
    image: Image,
    with_shader: bool,
}

impl common::Example for DrawImageExample {
    fn setup(ctx: &mut GraphicsContext) -> Self {
        let shader = Shader::new(
            ctx,
            wgpu::ShaderSource::Wgsl(include_str!("custom_red_only.wgsl").into()),
        );
        let image_bytes = include_bytes!("../resources/textures/happy-tree.png");
        let image_data = image::load_from_memory(image_bytes).unwrap();
        let image_rgba = image_data.as_rgba8().unwrap();
        let dimensions = image_data.dimensions();
        let image = Image::new(ctx, dimensions.0, dimensions.1, image_rgba);
        DrawImageExample {
            image,
            shader,
            with_shader: true,
        }
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
        if self.with_shader {
            oblivion::push_shader(render, &self.shader);
        }
        self.image.draw(render, Transform::default());
        if self.with_shader {
            oblivion::pop_shader(render);
        }
    }
}

fn main() {
    common::run::<DrawImageExample>();
}
