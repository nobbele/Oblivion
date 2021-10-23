use crate::{helpers::create_pipeline, GraphicsContext};

/// Shader object.
pub struct Shader {
    pub(crate) pipeline_id: usize,
}

impl Shader {
    pub fn new(ctx: &mut GraphicsContext, source: wgpu::ShaderSource) -> Self {
        ctx.pipeline_store.push(create_pipeline(
            &ctx.device,
            ctx.config.format,
            source,
            &ctx.texture_bind_group_layout,
            &ctx.mvp_bind_group_layout,
        ));
        Shader {
            pipeline_id: ctx.pipeline_store.len() - 1,
        }
    }
}
