use wgpu::{Device, RenderPipelineDescriptor, TextureFormat, VertexState};

use super::Pipeline;

pub struct MeshPipeline {
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Pipeline for MeshPipeline {
    fn apply<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
    }
}

// TODO Result
pub fn create_mesh_pipeline<'a>(
    device: &Device,
    format: TextureFormat,
    buffers: &[wgpu::VertexBufferLayout<'a>],
    source: wgpu::ShaderSource,
) -> MeshPipeline {
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Oblivion_MeshShader"),
        source,
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Oblivion_MeshRenderPipelineLayout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Oblivion_MeshRenderPipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "main",
            buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            clamp_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    });

    MeshPipeline { render_pipeline }
}
