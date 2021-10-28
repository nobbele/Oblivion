use std::rc::Rc;

use wgpu::util::DeviceExt;

use crate::{GraphicsContext, PipelineData, Render, Transform};

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
pub struct Image {
    data: PipelineData,
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
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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
            },
        }
    }

    /// Pushes this image to the draw queue.
    pub fn draw(&self, render: &mut Render, transform: Transform) {
        render.push_data(self.data.clone(), 1, transform, 0);
    }
}
