use std::{num::NonZeroU32, rc::Rc};

use wgpu::util::DeviceExt;

use crate::{GraphicsContext, OblivionError, OblivionResult, PipelineData, Render, Transform};

/// Essentially just a textured rectangle.
///
/// Example usage:
/// ```rust
/// let image_bytes = include_bytes!("../resources/textures/happy-tree.png");
/// let image_data = image::load_from_memory(image_bytes).unwrap();
/// let image_rgba = image_data.as_rgba8().unwrap();
/// let dimensions = image_data.dimensions();
/// let image = Image::new(ctx, [dimensions.0, dimensions.1], image_rgba);
/// /* ... */
/// image.draw(&mut render, Transform::default());
/// ```
#[derive(Clone)]
pub struct Image {
    data: PipelineData,
    dimensions: mint::Vector2<f32>,
    texture: Rc<wgpu::Texture>,
}

impl Image {
    /// Creates a new image object.
    pub fn new(
        ctx: &GraphicsContext,
        dimensions: impl Into<mint::Vector2<u32>>,
        data: &[u8],
    ) -> Self {
        let dimensions = dimensions.into();
        let size = wgpu::Extent3d {
            width: dimensions.x,
            height: dimensions.x,
            depth_or_array_layers: 1,
        };
        let texture = ctx.device.create_texture_with_data(
            &ctx.queue,
            &wgpu::TextureDescriptor {
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                label: Some("Oblivion_Texture"),
            },
            data,
        );

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

        Image {
            data: PipelineData {
                mesh_buffer: Rc::clone(&ctx.quad_mesh_buffer),
                bind_group: Rc::new(bind_group),
                instance_buffer: Rc::clone(&ctx.identity_instance_buffer),
                object_dimensions: mint::Vector2 { x: 1.0, y: 1.0 },
            },
            dimensions: ctx.gfx_config.render_dimensions,
            texture: Rc::new(texture),
        }
    }

    /// Gets the raw RGBA data of this canvas's underlying texture.
    pub fn download_rgba(&self, ctx: &mut GraphicsContext) -> OblivionResult<Vec<u8>> {
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as wgpu::BufferAddress;
        let byte_width = self.dimensions.x as u64 * 4;
        let padded_width_padding = (align - byte_width % align) % align;
        let padded_width = byte_width + padded_width_padding;

        let download_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Oblivion_ImageDownloadBuffer"),
            size: padded_width * self.dimensions.y as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut command_encoder =
            ctx.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Oblivion_ImageCommandEncoder"),
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
                    rows_per_image: NonZeroU32::new(self.dimensions.y as u32),
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
        let mut v = Vec::with_capacity(byte_width as usize * self.dimensions.y as usize);
        for y in 0..self.dimensions.y as u64 {
            let start = y as usize * padded_width as usize;
            v.extend_from_slice(&buffer_view[start..start + byte_width as usize]);
        }
        Ok(v)
    }

    /// Pushes this image to the draw queue.
    pub fn draw(&self, render: &mut Render, transform: Transform) {
        let mut transform = transform;
        transform.scale.x *= self.dimensions.x;
        transform.scale.y *= self.dimensions.y;
        render.push_data(self.data.clone(), 1, transform, 0);
    }
}
