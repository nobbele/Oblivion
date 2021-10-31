use std::{num::NonZeroU32, rc::Rc};

use glyph_brush::{ab_glyph::FontArc, GlyphBrush, GlyphBrushBuilder, Section};

use crate::{GraphicsContext, MeshBuffer, OblivionError, PipelineData, Render, Transform, Vertex};

pub struct TextFragment {
    text: String,
}

impl From<&str> for TextFragment {
    fn from(s: &str) -> Self {
        TextFragment { text: s.to_owned() }
    }
}

impl From<String> for TextFragment {
    fn from(s: String) -> Self {
        TextFragment { text: s }
    }
}

/// Renderable text object.
pub struct Text {
    pipeline_data: PipelineData,
    glyph_brush: GlyphBrush<[Vertex; 4], glyph_brush::Extra>,
    texture: wgpu::Texture,
    upload_buffer: wgpu::Buffer,
    upload_buffer_size: wgpu::BufferAddress,
    fragments: Vec<TextFragment>,
    dirty: bool,
}

impl Text {
    /// Creates a new text object.
    pub fn new(ctx: &mut GraphicsContext) -> Self {
        let mesh_buffer = MeshBuffer::from_slices(&ctx.device, &[], &[]);
        let default_font =
            FontArc::try_from_slice(include_bytes!("../../resources/fonts/DejaVuSans.ttf"))
                .map_err(OblivionError::LoadFont)
                .unwrap();
        let upload_buffer_size = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as wgpu::BufferAddress * 100;
        let upload_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Oblivion_TextUploadBuffer"),
            size: upload_buffer_size,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: false,
        });
        let glyph_brush = GlyphBrushBuilder::using_font(default_font).build();
        let texture_dimensions = glyph_brush.texture_dimensions();
        let texture_dimensions = [texture_dimensions.0, texture_dimensions.1];
        let (texture, bind_group) = create_texture(ctx, texture_dimensions);
        Text {
            pipeline_data: PipelineData {
                mesh_buffer: Rc::new(mesh_buffer),
                bind_group: Rc::new(bind_group),
                instance_buffer: Rc::clone(&ctx.identity_instance_buffer),
            },
            glyph_brush,
            texture,
            upload_buffer,
            upload_buffer_size,
            fragments: Vec::new(),
            dirty: false,
        }
    }

    // Make a add_text + flush method instead?
    /// Adds texts to the text object.
    pub fn add_text<const N: usize>(&mut self, frags: [impl Into<TextFragment>; N]) {
        self.fragments
            .extend(frags.map(|frag| frag.into()).into_iter());
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.fragments.clear();
    }

    pub fn flush(&mut self, ctx: &mut GraphicsContext) {
        for frag in &self.fragments {
            self.glyph_brush.queue(
                Section::default().add_text(glyph_brush::Text::new(&frag.text).with_scale(72.0)),
            );
        }

        match self.glyph_brush.process_queued(
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
                if self.upload_buffer_size < padded_data_size {
                    self.upload_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Oblivion_TextUploadBuffer"),
                        size: padded_data_size,
                        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
                        mapped_at_creation: false,
                    });

                    self.upload_buffer_size = padded_data_size;
                }

                let fut = self
                    .upload_buffer
                    .slice(..padded_data_size)
                    .map_async(wgpu::MapMode::Write);
                ctx.device.poll(wgpu::Maintain::Wait);
                pollster::block_on(fut)
                    .map_err(OblivionError::MapBuffer)
                    .unwrap();

                for row in 0..height {
                    self.upload_buffer
                        .slice(row * padded_width..(row + 1) * padded_width)
                        .get_mapped_range_mut()[0..width as usize]
                        .copy_from_slice(
                            &tex_data[row as usize * width as usize
                                ..(row as usize + 1) * width as usize],
                        )
                }

                self.upload_buffer.unmap();

                let mut command_encoder =
                    ctx.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Oblivion_TextCommandEncoder"),
                        });
                command_encoder.copy_buffer_to_texture(
                    wgpu::ImageCopyBuffer {
                        buffer: &self.upload_buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: NonZeroU32::new(padded_width as u32),
                            rows_per_image: NonZeroU32::new(height as u32),
                        },
                    },
                    wgpu::ImageCopyTexture {
                        texture: &self.texture,
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
                        .map(|vertex_list| {
                            vertex_list.map(|mut v| {
                                v.position = [
                                    v.position.x / 900.0,
                                    (v.position.y - 72.0 / 2.0) / 900.0 + 0.5,
                                ]
                                .into();
                                v
                            })
                        })
                        .flatten()
                        .collect::<Vec<_>>();
                    let mesh_buffer = MeshBuffer::from_slices(&ctx.device, &vertices, &indices);
                    self.pipeline_data.mesh_buffer = Rc::new(mesh_buffer);
                }
                glyph_brush::BrushAction::ReDraw => {}
            },
            Err(e) => match e {
                glyph_brush::BrushError::TextureTooSmall { suggested } => {
                    let new_dimensions = [suggested.0, suggested.1];
                    self.resize_texture(ctx, new_dimensions);
                }
            },
        };
        self.dirty = false;
    }

    fn resize_texture(&mut self, ctx: &mut GraphicsContext, dimensions: [u32; 2]) {
        let (texture, bind_group) = create_texture(ctx, dimensions);
        self.texture = texture;
        self.pipeline_data.bind_group = Rc::new(bind_group);
    }

    /// Pushes this text object to the draw queue.
    pub fn draw(&self, render: &mut Render, transform: Transform) {
        if self.dirty {
            panic!("Call Text::flush before draw!");
        }
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
