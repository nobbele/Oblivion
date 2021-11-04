use image::GenericImageView;
use oblivion::{GraphicsContext, Image, Render, Shader, Transform};
#[path = "common.rs"]
mod common;

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
        let image = Image::new(ctx, [dimensions.0, dimensions.1], image_rgba);
        DrawImageExample {
            image,
            shader,
            with_shader: true,
        }
    }

    fn draw(&self, render: &mut Render) {
        oblivion::clear(render, [0.1, 0.2, 0.3, 1.0]);
        if self.with_shader {
            oblivion::push_shader(render, &self.shader);
        }
        oblivion::set_shader_data(render, &[1.0f32, 0.0, 1.0, 1.0]);
        self.image.draw(
            render,
            Transform {
                position: [0.25, 0.5].into(),
                scale: [0.5, 0.5].into(),
                ..Default::default()
            },
        );
        oblivion::set_shader_data(render, &[0.0f32, 1.0, 1.0, 1.0]);
        self.image.draw(
            render,
            Transform {
                position: [0.75, 0.5].into(),
                scale: [0.5, 0.5].into(),
                ..Default::default()
            },
        );
        if self.with_shader {
            oblivion::pop_shader(render);
        }
    }
}

fn main() {
    common::run::<DrawImageExample>();
}
