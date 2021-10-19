#![allow(dead_code)]
use wgpu::{Device, RenderPipelineDescriptor, TextureFormat, VertexState};

use super::Pipeline;

pub struct CustomPipeline {
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl Pipeline for CustomPipeline {
    fn apply<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
    }
}

#[derive(Clone, Copy)]
pub struct TextureConfig {}

// TODO Result
pub fn create_custom_pipeline<'a>(
    device: &Device,
    format: TextureFormat,
    texture_config: Option<TextureConfig>,
    buffers: &[wgpu::VertexBufferLayout<'a>],
    source: wgpu::ShaderSource,
) -> CustomPipeline {
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Oblivion_CustomShader"),
        source,
    });

    let bind_group_layout = texture_config.map(|_texture_config| {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            ],
            label: Some("Oblivion_CustomBindGroupLayout"),
        })
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Oblivion_CustomRenderPipelineLayout"),
        bind_group_layouts: if let Some(ref a) = bind_group_layout.as_ref() {
            std::slice::from_ref(a)
        } else {
            &[]
        },
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Oblivion_CustomRenderPipeline"),
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

    CustomPipeline {
        render_pipeline,
        bind_group_layout,
    }
}
