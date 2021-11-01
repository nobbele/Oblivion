use std::{num::NonZeroU64, rc::Rc};

use glyph_brush::{GlyphBrush, GlyphBrushBuilder};
use wgpu::util::DeviceExt;

use crate::{
    helpers::{create_pipeline, get_adapter_surface, get_device_queue},
    internal::PipelineData,
    DrawData, Font, MeshBuffer, OblivionError, OblivionResult, Render, RenderData, RenderGroup,
    TargetId, Transform, Vertex, QUAD_INDICES, QUAD_VERTICES,
};

type UniformType = [[f32; 4]; 4];
const UNIFORM_SIZE: usize = std::mem::size_of::<UniformType>();

/// Context for graphics. This stores the graphics device, render queue, window surface, and more.
pub struct GraphicsContext {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface: wgpu::Surface,
    pub(crate) preferred_format: wgpu::TextureFormat,
    #[allow(dead_code)]
    pub(crate) config: wgpu::SurfaceConfiguration,

    pub(crate) canvas_store: Vec<wgpu::TextureView>,
    pub(crate) pipeline_store: Vec<wgpu::RenderPipeline>,
    pub(crate) default_font: Font,
    pub(crate) glyph_brush: GlyphBrush<[Vertex; 4]>,

    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) mvp_bind_group_layout: wgpu::BindGroupLayout,

    pub(crate) quad_mesh_buffer: Rc<MeshBuffer>,
    pub(crate) identity_instance_buffer: Rc<wgpu::Buffer>,

    projection: glam::Mat4,

    uniform_alignment: u32,
    uniform_buffer_data: Vec<u8>,
    uniform_buffer: wgpu::Buffer,
    uniform_buffer_count: u64,

    uniform_bind_groups: Vec<wgpu::BindGroup>,
}

impl GraphicsContext {
    /// Creates a new graphics context.
    /// The window parameter can be a winit window or similar.
    pub fn new(
        window: &impl raw_window_handle::HasRawWindowHandle,
        dimensions: impl Into<mint::Vector2<u32>>,
        vsync: bool,
    ) -> OblivionResult<Self> {
        let dimensions = dimensions.into();
        let (adapter, surface) = get_adapter_surface(window)?;
        let (device, queue) = get_device_queue(&adapter)?;

        let preferred_format = surface
            .get_preferred_format(&adapter)
            .ok_or(OblivionError::InvalidSurface)?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: preferred_format,
            width: dimensions.x,
            height: dimensions.y,
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
            "Standard",
            &device,
            config.format,
            wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/shader.wgsl").into()),
            &texture_bind_group_layout,
            &mvp_bind_group_layout,
        );
        let text_pipeline = create_pipeline(
            "Text",
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

        let identity_instance_buffer = Rc::new(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Oblivion_IdentityInstanceBuffer"),
                usage: wgpu::BufferUsages::VERTEX,
                contents: bytemuck::cast_slice(&[Transform {
                    offset: [0.0, 0.0].into(),
                    ..Default::default()
                }
                .as_matrix(mint::Vector2 { x: 1.0, y: 1.0 })
                .to_cols_array_2d()]),
            }),
        );

        // ?? Stolen from ggez
        fn ortho(
            left: f32,
            right: f32,
            top: f32,
            bottom: f32,
            far: f32,
            near: f32,
        ) -> [[f32; 4]; 4] {
            let c0r0 = 2.0 / (right - left);
            let c0r1 = 0.0;
            let c0r2 = 0.0;
            let c0r3 = 0.0;

            let c1r0 = 0.0;
            let c1r1 = 2.0 / (top - bottom);
            let c1r2 = 0.0;
            let c1r3 = 0.0;

            let c2r0 = 0.0;
            let c2r1 = 0.0;
            let c2r2 = -2.0 / (far - near);
            let c2r3 = 0.0;

            let c3r0 = -(right + left) / (right - left);
            let c3r1 = -(top + bottom) / (top - bottom);
            let c3r2 = -(far + near) / (far - near);
            let c3r3 = 1.0;

            // our matrices are column-major, so here we are.
            [
                [c0r0, c0r1, c0r2, c0r3],
                [c1r0, c1r1, c1r2, c1r3],
                [c2r0, c2r1, c2r2, c2r3],
                [c3r0, c3r1, c3r2, c3r3],
            ]
        }
        let projection = glam::Mat4::from_cols_array_2d(&ortho(0.0, 1.0, 0.0, 1.0, -1.0, 1.0));

        let mut glyph_brush = GlyphBrushBuilder::using_fonts(vec![]).build();
        let default_font = Font::new_raw(
            &mut glyph_brush,
            include_bytes!("../resources/fonts/DejaVuSans.ttf").to_vec(),
        )?;

        Ok(GraphicsContext {
            device,
            queue,
            surface,
            preferred_format,
            config,
            canvas_store: Vec::new(),
            pipeline_store,

            texture_bind_group_layout,
            mvp_bind_group_layout,

            glyph_brush,
            default_font,

            quad_mesh_buffer: Rc::new(quad_mesh_buffer),

            identity_instance_buffer,

            projection,
            uniform_alignment,
            uniform_buffer_data: Vec::new(),
            uniform_buffer,
            uniform_buffer_count: 0,
            uniform_bind_groups: Vec::new(),
        })
    }

    fn render_group(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: &wgpu::TextureView,
        group: &RenderGroup,
        uniform_start_idx: usize,
    ) {
        let uniform_alignment = self.uniform_alignment as wgpu::BufferAddress;
        let view = match group.target_id {
            TargetId::Screen => output_view,
            TargetId::CanvasId(canvas_id) => &self.canvas_store[canvas_id],
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Oblivion_RenderPass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: match group.clear_color {
                        Some(color) => wgpu::LoadOp::Clear(wgpu::Color {
                            r: color.r as _,
                            g: color.g as _,
                            b: color.b as _,
                            a: color.a as _,
                        }),
                        None => wgpu::LoadOp::Load,
                    },
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        if !group.queue.is_empty() {
            for (
                idx,
                RenderData {
                    instance_data: DrawData { transform, .. },
                    pipeline_data:
                        PipelineData {
                            object_dimensions, ..
                        },
                    ..
                },
            ) in group.queue.iter().enumerate()
            {
                let start = (idx + uniform_start_idx) * uniform_alignment as usize;
                let mat = self.projection * transform.as_matrix(*object_dimensions);
                self.uniform_buffer_data[start..start + UNIFORM_SIZE]
                    .copy_from_slice(bytemuck::cast_slice(&mat.to_cols_array_2d()))
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
            ) in group.queue.iter().enumerate()
            {
                //println!("Drawing pipeline {}", *pipeline_id);
                render_pass.set_pipeline(&self.pipeline_store[*pipeline_id]);
                render_pass.set_bind_group(0, &pipeline_data.bind_group, &[]);
                render_pass.set_bind_group(
                    1,
                    &self.uniform_bind_groups[uniform_start_idx + idx],
                    &[],
                );
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

    // Maybe take render as &mut?
    /// Submits the render object to Oblivion's rendering system.
    pub fn submit_render(&mut self, render: Render) -> OblivionResult<()> {
        //println!("Starting render!");
        let uniform_alignment = self.uniform_alignment as wgpu::BufferAddress;
        let output = self
            .surface
            .get_current_texture()
            .map_err(OblivionError::RetrieveFrameError)?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let total_queue_len: u64 = render
            .render_groups
            .iter()
            .map(|group| group.queue.len() as u64)
            .sum();
        if total_queue_len > self.uniform_buffer_count {
            let new_uniform_buffer_count = total_queue_len * 2;
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
                                size: Some(
                                    NonZeroU64::new(uniform_alignment)
                                        .expect("This is definitely not zero"),
                                ),
                            }),
                        }],
                        label: Some("Oblivion_MVPBindGroup"),
                    })
                })
                .collect::<Vec<_>>();
            self.uniform_buffer_data =
                vec![0; new_uniform_buffer_count as usize * uniform_alignment as usize];

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

        // Surface frame times out if there's no render pass used.
        if render.render_groups.is_empty() {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Oblivion_RenderPass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        let mut uniform_start_idx = 0;
        for group in &render.render_groups {
            /*println!(
                "Group Target: {:?} ({} draws)",
                group.target_id,
                group.queue.len()
            );*/
            self.render_group(&mut encoder, &view, group, uniform_start_idx);
            uniform_start_idx += group.queue.len();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        //println!("Render finished!");
        Ok(())
    }
}
