use std::rc::Rc;

use crate::{
    helpers::{create_pipeline, get_adapter_surface, get_device_queue},
    InstanceData, MeshBuffer, Render, RenderData, QUAD_INDICES, QUAD_VERTICES,
};

pub struct GraphicsContext {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    surface: wgpu::Surface,
    #[allow(dead_code)]
    pub(crate) config: wgpu::SurfaceConfiguration,

    pub(crate) pipeline_store: Vec<wgpu::RenderPipeline>,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) mvp_bind_group_layout: wgpu::BindGroupLayout,

    pub(crate) quad_mesh_buffer: Rc<MeshBuffer>,
}

impl GraphicsContext {
    // TODO Result
    pub fn new(
        window: &impl raw_window_handle::HasRawWindowHandle,
        width: u32,
        height: u32,
        vsync: bool,
    ) -> Self {
        let (adapter, surface) = get_adapter_surface(window);
        let (device, queue) = get_device_queue(&adapter);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width,
            height,
            present_mode: if vsync {
                wgpu::PresentMode::Fifo
            } else {
                wgpu::PresentMode::Immediate
            },
        };

        surface.configure(&device, &config);

        let texture_bind_group_layout =
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
                label: Some("Oblivion_TextureBindGroupLayout"),
            });
        let mvp_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Oblivion_MVPBindGroupLayout"),
            });

        let standard_pipeline = create_pipeline(
            &device,
            config.format,
            wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/shader.wgsl").into()),
            &texture_bind_group_layout,
            &mvp_bind_group_layout,
        );

        let pipeline_store = vec![standard_pipeline];

        let quad_mesh_buffer = MeshBuffer::from_slices(&device, QUAD_VERTICES, QUAD_INDICES);

        GraphicsContext {
            surface,
            device,
            queue,
            config,
            pipeline_store,
            quad_mesh_buffer: Rc::new(quad_mesh_buffer),
            texture_bind_group_layout,
            mvp_bind_group_layout,
        }
    }

    // TODO Result
    pub fn submit_render(&self, mut render: Render) {
        let output = self
            .surface
            .get_current_texture()
            .expect("Unable to get surface texture");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Oblivion_CommandEncoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Oblivion_RenderPass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match render.clear_color {
                            Some(color) => wgpu::LoadOp::Clear(color),
                            None => wgpu::LoadOp::Load,
                        },
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            for RenderData {
                pipeline_data,
                instance_data:
                    InstanceData {
                        pipeline_id,
                        transform,
                    },
            } in &render.queue
            {
                render_pass.set_pipeline(&self.pipeline_store[*pipeline_id]);
                render_pass.set_bind_group(0, &pipeline_data.bind_group, &[]);
                render_pass.set_bind_group(1, &transform, &[]);
                render_pass.set_vertex_buffer(0, pipeline_data.mesh_buffer.vertex.0.slice(..));
                render_pass.set_index_buffer(
                    pipeline_data.mesh_buffer.index.0.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(0..pipeline_data.mesh_buffer.index.1, 0, 0..1);
            }
        }
        render.queue.clear();

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
