use std::{num::NonZeroU32, rc::Rc};

use glyph_brush::{ab_glyph::FontArc, GlyphBrushBuilder, Section};

use crate::{GraphicsContext, MeshBuffer, PipelineData, Render, Transform, Vertex};

pub struct Text {
    pipeline_data: Rc<PipelineData>,
}

impl Text {
    // TODO make this better
    pub fn new(ctx: &mut GraphicsContext) -> Self {
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Oblivion_TextTexture"),
            // TODO how to dynamically size this?
            size: wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            mip_level_count: 1,
            sample_count: 1,
        });

        // Maybe use Queue::write_texture instead?
        let mut upload_buffer_size =
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as wgpu::BufferAddress * 100;
        let mut upload_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Oblivion_TextUploadBuffer"),
            size: upload_buffer_size,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: false,
        });

        let font = FontArc::try_from_slice(include_bytes!("../../resources/fonts/DejaVuSans.ttf"))
            .unwrap();
        let mut glyph_brush = GlyphBrushBuilder::using_font(font).build();
        glyph_brush.queue(
            Section::default().add_text(glyph_brush::Text::new("Hello World").with_scale(48.0)),
        );
        let mesh_buffer = match glyph_brush.process_queued(
            |rect, tex_data| {
                let [width, height] = [
                    rect.width() as wgpu::BufferAddress,
                    rect.height() as wgpu::BufferAddress,
                ];
                let offset = [
                    rect.min[0] as wgpu::BufferAddress,
                    rect.min[1] as wgpu::BufferAddress,
                ];

                let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as wgpu::BufferAddress;
                let padded_width_padding = (align - width % align) % align;
                let padded_width = width + padded_width_padding;

                let padded_data_size = padded_width * height;
                if upload_buffer_size < padded_data_size {
                    upload_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Oblivion_TextUploadBuffer"),
                        size: padded_data_size,
                        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
                        mapped_at_creation: false,
                    });

                    upload_buffer_size = padded_data_size;
                }

                let fut = upload_buffer.slice(..).map_async(wgpu::MapMode::Write);
                ctx.device.poll(wgpu::Maintain::Wait);
                pollster::block_on(fut).unwrap();

                for row in 0..height {
                    upload_buffer
                        .slice(row * padded_width..row * padded_width + width)
                        .get_mapped_range_mut()
                        .copy_from_slice(
                            &tex_data[row as usize * width as usize
                                ..(row as usize + 1) * width as usize],
                        )
                }

                upload_buffer.unmap();

                let mut command_encoder =
                    ctx.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Oblivion_TextCommandEncoder"),
                        });
                command_encoder.copy_buffer_to_texture(
                    wgpu::ImageCopyBuffer {
                        buffer: &upload_buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: NonZeroU32::new(padded_width as u32),
                            rows_per_image: NonZeroU32::new(height as u32),
                        },
                    },
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: offset[0] as u32,
                            y: offset[1] as u32,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: width as u32,
                        height: height as u32,
                        depth_or_array_layers: 1,
                    },
                );
                ctx.queue.submit(std::iter::once(command_encoder.finish()));
            },
            |vertex_data| {
                let color = [
                    vertex_data.extra.color[0],
                    vertex_data.extra.color[1],
                    vertex_data.extra.color[2],
                ];
                let mut pixel_coords: [[f32; 2]; 4] = [
                    [
                        vertex_data.pixel_coords.min.x,
                        vertex_data.pixel_coords.min.y,
                    ],
                    [
                        vertex_data.pixel_coords.max.x,
                        vertex_data.pixel_coords.min.y,
                    ],
                    [
                        vertex_data.pixel_coords.max.x,
                        vertex_data.pixel_coords.max.y,
                    ],
                    [
                        vertex_data.pixel_coords.min.x,
                        vertex_data.pixel_coords.max.y,
                    ],
                ];
                for i in 0..pixel_coords.len() {
                    // TODO fix this
                    pixel_coords[i] = [
                        pixel_coords[i][0] / 200.0,
                        1.0 - (pixel_coords[i][1] / 200.0),
                    ]
                }
                let uv_coords: [[f32; 2]; 4] = [
                    [vertex_data.tex_coords.min.x, vertex_data.tex_coords.min.y],
                    [vertex_data.tex_coords.max.x, vertex_data.tex_coords.min.y],
                    [vertex_data.tex_coords.max.x, vertex_data.tex_coords.max.y],
                    [vertex_data.tex_coords.min.x, vertex_data.tex_coords.max.y],
                ];
                [
                    Vertex {
                        position: pixel_coords[0],
                        color,
                        uv: uv_coords[0],
                    },
                    Vertex {
                        position: pixel_coords[1],
                        color,
                        uv: uv_coords[1],
                    },
                    Vertex {
                        position: pixel_coords[2],
                        color,
                        uv: uv_coords[2],
                    },
                    Vertex {
                        position: pixel_coords[3],
                        color,
                        uv: uv_coords[3],
                    },
                ]
            },
        ) {
            Ok(action) => match action {
                glyph_brush::BrushAction::Draw(vertices_list) => {
                    let indices = vertices_list
                        .iter()
                        .scan(0, |base_index, _| {
                            let indices = [
                                *base_index,
                                *base_index + 1,
                                *base_index + 2,
                                *base_index,
                                *base_index + 2,
                                *base_index + 3,
                            ];
                            *base_index += 4;
                            Some(indices)
                        })
                        .flatten()
                        .collect::<Vec<_>>();
                    let vertices = vertices_list.into_iter().flatten().collect::<Vec<_>>();
                    MeshBuffer::from_slices(&ctx.device, &vertices, &indices)
                }
                glyph_brush::BrushAction::ReDraw => todo!(),
            },
            r => panic!("{:#?}", r),
        };

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &ctx.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Oblivion_ImageBindGroup"),
        });

        Text {
            pipeline_data: Rc::new(PipelineData {
                mesh_buffer: Rc::new(mesh_buffer),
                bind_group,
            }),
        }
    }

    pub fn draw(&self, render: &mut Render, transform: Transform) {
        render.push_data(self.pipeline_data.clone(), transform, 1);
    }
}
