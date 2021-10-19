pub mod custom_pipeline;
pub mod mesh_pipeline;
pub mod texture_pipeline;

pub trait Pipeline {
    fn apply<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}
