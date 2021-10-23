use pollster::block_on;
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, Limits, Queue, RequestAdapterOptions,
    Surface,
};

use crate::{instance_desc, Vertex};

// TODO Result
pub fn get_adapter_surface(
    window: &impl raw_window_handle::HasRawWindowHandle,
) -> (Adapter, Surface) {
    let instance = wgpu::Instance::new(Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(window) };
    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .expect("Unable to find adapter");

    (adapter, surface)
}

// TODO Result
pub fn get_device_queue(adapter: &Adapter) -> (Device, Queue) {
    block_on(adapter.request_device(
        &DeviceDescriptor {
            label: Some("Oblivion_Device"),
            features: Features::default(),
            limits: Limits::default(),
        },
        None,
    ))
    .expect("Unable to create device")
}

pub fn create_pipeline(
    device: &Device,
    format: wgpu::TextureFormat,
    source: wgpu::ShaderSource,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    mvp_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Oblivion_Shader"),
        source,
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Oblivion_RenderPipelineLayout"),
        bind_group_layouts: &[texture_bind_group_layout, mvp_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Oblivion_RenderPipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "main",
            buffers: &[Vertex::desc(), instance_desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: None, //Some(wgpu::Face::Back),
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

    render_pipeline
}
