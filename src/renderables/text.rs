use std::{num::NonZeroU32, rc::Rc};

use glyph_brush::{ab_glyph::FontArc, GlyphBrushBuilder, GlyphCruncher, Section};

use crate::{GraphicsContext, MeshBuffer, PipelineData, Render, Transform, Vertex};

/// Renderable text object.
pub struct Text {
    pipeline_data: PipelineData,
}

impl Text {
    pub fn new(ctx: &mut GraphicsContext) -> Self {
        let (_texture, bind_group) = create_texture(ctx, [1, 1]);
        let mesh_buffer = MeshBuffer::from_slices(&ctx.device, &[], &[]);
        Text {
            pipeline_data: PipelineData {
                mesh_buffer: Rc::new(mesh_buffer),
                bind_group: Rc::new(bind_group),
                instance_buffer: Rc::clone(&ctx.identity_instance_buffer),
            },
        }
    }

    // Make a add_text + flush method instead?
    pub fn add_text(&mut self, ctx: &mut GraphicsContext, texts: &[&str]) {
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
        let sections = texts
            .iter()
            .map(|&text| Section::default().add_text(glyph_brush::Text::new(text).with_scale(72.0)))
            .collect::<Vec<_>>();
        for section in &sections {
            glyph_brush.queue(section);
        }

        let texture_dimensions = glyph_brush.texture_dimensions();
        let (texture, bind_group) =
            create_texture(ctx, [texture_dimensions.0, texture_dimensions.1]);
        self.pipeline_data.bind_group = Rc::new(bind_group);

        let max_point = sections
            .iter()
            .filter_map(|section| glyph_brush.glyph_bounds(section))
            .map(|bounds| bounds.max)
            .reduce(|a, b| glyph_brush::ab_glyph::Point {
                x: a.x.max(b.x),
                y: a.y.max(b.y),
            })
            .unwrap();

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
                let color = <[f32; 3]>::try_from(&vertex_data.extra.color[0..3])
                    .unwrap()
                    .into();
                let pixel_coords = glyph_rect_to_point_list(vertex_data.pixel_coords);
                let uv_coords = glyph_rect_to_point_list(vertex_data.tex_coords);
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
                    let vertices = vertices_list
                        .into_iter()
                        .flatten()
                        .map(|mut v| {
                            v.position = [
                                (v.position.x) / max_point.x * 2.0 - 1.0,
                                (v.position.y - 72.0 / 2.0) / max_point.x * -2.0,
                            ]
                            .into();
                            v
                        })
                        .collect::<Vec<_>>();
                    MeshBuffer::from_slices(&ctx.device, &vertices, &indices)
                }
                glyph_brush::BrushAction::ReDraw => todo!(),
            },
            Err(e) => panic!("{:#?}", e),
        };
        self.pipeline_data.mesh_buffer = Rc::new(mesh_buffer);
    }

    pub fn draw(&self, render: &mut Render, transform: Transform) {
        render.push_data(self.pipeline_data.clone(), 1, transform, 1);
    }
}

fn create_texture(
    ctx: &mut GraphicsContext,
    dimensions: [u32; 2],
) -> (wgpu::Texture, wgpu::BindGroup) {
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Oblivion_TextTexture"),
        size: wgpu::Extent3d {
            width: dimensions[0],
            height: dimensions[1],
            depth_or_array_layers: 1,
        },
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        mip_level_count: 1,
        sample_count: 1,
    });

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

    (texture, bind_group)
}

fn glyph_rect_to_point_list(rect: glyph_brush::ab_glyph::Rect) -> [mint::Point2<f32>; 4] {
    [
        [rect.min.x, rect.min.y].into(),
        [rect.max.x, rect.min.y].into(),
        [rect.max.x, rect.max.y].into(),
        [rect.min.x, rect.max.y].into(),
    ]
}
