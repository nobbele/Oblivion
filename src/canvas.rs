use std::{num::NonZeroU32, rc::Rc};

use crate::{GraphicsContext, OblivionError, OblivionResult, PipelineData, Render, Transform};

// TODO make this a wrapper of image maybe?

/// Canvases are used as rendering target to create a fake screen.
pub struct Canvas {
    pub(crate) canvas_id: usize,
    pub(crate) texture: wgpu::Texture,
    pub(crate) data: PipelineData,
    pub dimensions: mint::Vector2<u32>,
}

impl Canvas {
    /// Creates a new canvas.
    pub fn new(ctx: &mut GraphicsContext, dimensions: impl Into<mint::Vector2<u32>>) -> Self {
        let dimensions = dimensions.into();
        let size = wgpu::Extent3d {
            width: dimensions.x,
            height: dimensions.y,
            depth_or_array_layers: 1,
        };
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ctx.preferred_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            label: Some("Oblivion_Texture"),
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
            label: Some("Oblivion_CanvasBindGroup"),
        });

        ctx.canvas_store.push(texture_view);
        let id = ctx.canvas_store.len() - 1;
        Canvas {
            canvas_id: id,
            data: PipelineData {
                mesh_buffer: Rc::clone(&ctx.quad_mesh_buffer),
                bind_group: Rc::new(bind_group),
                instance_buffer: Rc::clone(&ctx.identity_instance_buffer),
            },
            texture,
            dimensions,
        }
    }

    /// Gets the raw RGBA data of this canvas's underlying texture.
    pub fn download_rgba(&self, ctx: &mut GraphicsContext) -> OblivionResult<Vec<u8>> {
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as wgpu::BufferAddress;
        let padded_width_padding = (align - self.dimensions.x as u64 % align) % align;
        let padded_width = self.dimensions.x as u64 + padded_width_padding;

        let download_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Oblivion_TextUploadBuffer"),
            size: padded_width * self.dimensions.y as u64 * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut command_encoder =
            ctx.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Oblivion_TextCommandEncoder"),
                });
        command_encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &download_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(padded_width as u32),
                    rows_per_image: NonZeroU32::new(self.dimensions.x as u32),
                },
            },
            wgpu::Extent3d {
                width: self.dimensions.x as u32,
                height: self.dimensions.y as u32,
                depth_or_array_layers: 1,
            },
        );
        ctx.queue.submit(std::iter::once(command_encoder.finish()));

        let fut = download_buffer.slice(..).map_async(wgpu::MapMode::Read);
        ctx.device.poll(wgpu::Maintain::Wait);
        pollster::block_on(fut).map_err(OblivionError::MapBuffer)?;

        let buffer_view = download_buffer.slice(..).get_mapped_range();
        let mut v = Vec::with_capacity(self.dimensions.x as usize * self.dimensions.y as usize * 4);
        for y in 0..self.dimensions.y as u64 {
            let start = y as usize * padded_width as usize;
            v.extend_from_slice(&buffer_view[start..start + self.dimensions.x as usize * 4]);
        }
        Ok(v)
    }

    /// Pushes this canvas to the draw queue.
    pub fn draw(&self, render: &mut Render, transform: Transform) {
        render.push_data(self.data.clone(), 1, transform, 0);
    }
}
