use crate::{pipelines::custom_pipeline::create_custom_pipeline, GraphicsContext};

pub struct CustomShader {}

impl CustomShader {
    pub fn new(ctx: &mut GraphicsContext, source: wgpu::ShaderSource) -> Self {
        let custom_pipeline =
            create_custom_pipeline(&ctx.device, ctx.config.format, None, &[], source);
        CustomShader {}
    }
}
