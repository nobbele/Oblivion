use std::{num::NonZeroU64, rc::Rc};

use wgpu::util::DeviceExt;

use crate::{
    helpers::{create_pipeline, get_adapter_surface, get_device_queue},
    DrawData, MeshBuffer, Render, RenderData, Transform, QUAD_INDICES, QUAD_VERTICES,
};

type UniformType = [[f32; 4]; 4];
const UNIFORM_SIZE: usize = std::mem::size_of::<UniformType>();

/// Context for graphics. This stores the graphics device, render queue, window surface, and more.
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
    pub(crate) identity_instance_buffer: Rc<wgpu::Buffer>,

    uniform_alignment: u32,
    uniform_buffer_data: Vec<u8>,
    uniform_buffer: wgpu::Buffer,
    uniform_buffer_count: u64,

    uniform_bind_groups: Vec<wgpu::BindGroup>,
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
        let text_pipeline = create_pipeline(
            &device,
            config.format,
            wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/text_shader.wgsl").into()),
            &texture_bind_group_layout,
            &mvp_bind_group_layout,
        );

        let pipeline_store = vec![standard_pipeline, text_pipeline];

        let quad_mesh_buffer = MeshBuffer::from_slices(&device, QUAD_VERTICES, QUAD_INDICES);

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Oblivion_UniformBuffer"),
            size: 0,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_alignment = device.limits().min_uniform_buffer_offset_alignment;

        let identity_instance_buffer = Rc::new(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Oblivion_IdentityInstanceBuffer"),
                usage: wgpu::BufferUsages::VERTEX,
                contents: bytemuck::cast_slice(&[
                    Transform::default().as_matrix().to_cols_array_2d(),
                ]),
            },
        ));

        GraphicsContext {
            surface,
            device,
            queue,
            config,
            pipeline_store,
            quad_mesh_buffer: Rc::new(quad_mesh_buffer),
            texture_bind_group_layout,
            mvp_bind_group_layout,

            identity_instance_buffer,

            uniform_buffer,
            uniform_buffer_data: Vec::new(),
            uniform_buffer_count: 0,
            uniform_bind_groups: Vec::new(),
            uniform_alignment,
        }
    }

    // TODO Result
    pub fn submit_render(&mut self, mut render: Render) {
        let uniform_alignment = self.uniform_alignment as wgpu::BufferAddress;
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        if render.queue.len() as u64 > self.uniform_buffer_count {
            let new_uniform_buffer_count = render.queue.len() as u64 * 2;
            self.uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Oblivion_UniformBuffer"),
                size: new_uniform_buffer_count * uniform_alignment,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.uniform_bind_groups = (0..new_uniform_buffer_count)
                .map(|idx| {
                    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &self.mvp_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.uniform_buffer,
                                offset: idx * uniform_alignment,
                                size: Some(NonZeroU64::new(uniform_alignment).unwrap()),
                            }),
                        }],
                        label: Some("Oblivion_MVPBindGroup"),
                    })
                })
                .collect::<Vec<_>>();
            self.uniform_buffer_data = vec![0; render.queue.len() * uniform_alignment as usize];

            println!(
                "New Uniform Buffer Size: {} -> {}!",
                self.uniform_buffer_count, new_uniform_buffer_count
            );
            self.uniform_buffer_count = new_uniform_buffer_count;
        }

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

            if !render.queue.is_empty() {
                for (
                    idx,
                    RenderData {
                        instance_data: DrawData { transform, .. },
                        ..
                    },
                ) in render.queue.iter().enumerate()
                {
                    let start = idx * uniform_alignment as usize;
                    self.uniform_buffer_data[start..start + UNIFORM_SIZE].copy_from_slice(
                        bytemuck::cast_slice(&transform.as_matrix().to_cols_array_2d()),
                    )
                }
                self.queue
                    .write_buffer(&self.uniform_buffer, 0, &self.uniform_buffer_data);

                for (
                    idx,
                    RenderData {
                        pipeline_data,
                        instance_count,
                        instance_data: DrawData { pipeline_id, .. },
                    },
                ) in render.queue.iter().enumerate()
                {
                    render_pass.set_pipeline(&self.pipeline_store[*pipeline_id]);
                    render_pass.set_bind_group(0, &pipeline_data.bind_group, &[]);
                    render_pass.set_bind_group(1, &self.uniform_bind_groups[idx], &[]);
                    render_pass.set_vertex_buffer(0, pipeline_data.mesh_buffer.vertex.0.slice(..));
                    render_pass.set_vertex_buffer(1, pipeline_data.instance_buffer.slice(..));
                    render_pass.set_index_buffer(
                        pipeline_data.mesh_buffer.index.0.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    render_pass.draw_indexed(
                        0..pipeline_data.mesh_buffer.index.1,
                        0,
                        0..*instance_count,
                    );
                }
            }
        }
        render.queue.clear();

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
