use std::rc::Rc;

use wgpu::util::DeviceExt;

use crate::{GraphicsContext, PipelineData, Render, Transform, INSTANCE_SIZE};

/// Draws a single texture using many `Transform`s
pub struct ImageBatch {
    data: PipelineData,
    instance_buffer_capacity: u64,
    instance_buffer_count: u64,
}

impl ImageBatch {
    /// Creates a new image batch object.
    pub fn new(
        ctx: &GraphicsContext,
        dimensions: impl Into<mint::Vector2<u32>>,
        data: &[u8],
    ) -> Self {
        let dimensions = dimensions.into();
        let size = wgpu::Extent3d {
            width: dimensions.x,
            height: dimensions.y,
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

        let instance_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
            size: 0,
            mapped_at_creation: false,
        });
        let instance_buffer = Rc::new(instance_buffer);

        ImageBatch {
            data: PipelineData {
                mesh_buffer: Rc::clone(&ctx.quad_mesh_buffer),
                bind_group: Rc::new(bind_group),
                instance_buffer: Rc::clone(&instance_buffer),
                object_dimensions: mint::Vector2 { x: 1.0, y: 1.0 },
            },
            instance_buffer_capacity: 0,
            instance_buffer_count: 0,
        }
    }

    /// Adds batch instances to the image batch.
    pub fn add_instance(&mut self, ctx: &mut GraphicsContext, transforms: &[Transform]) {
        let new_count = self.instance_buffer_count + transforms.len() as u64;
        if new_count > self.instance_buffer_capacity {
            let new_capacity = new_count;
            let instance_buffer = Rc::new(ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Oblivion_ImageBatchInstanceBuffer"),
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                size: new_capacity * INSTANCE_SIZE as u64,
                mapped_at_creation: false,
            }));

            let mut command_encoder =
                ctx.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Oblivion_ImageBatchInstanceBufferResizeCopyCommandEncoder"),
                    });
            command_encoder.copy_buffer_to_buffer(
                &self.data.instance_buffer,
                0,
                &instance_buffer,
                0,
                self.instance_buffer_count * INSTANCE_SIZE as u64,
            );
            ctx.queue.submit(std::iter::once(command_encoder.finish()));

            self.data = PipelineData {
                instance_buffer: Rc::clone(&instance_buffer),
                mesh_buffer: Rc::clone(&self.data.mesh_buffer),
                bind_group: Rc::clone(&self.data.bind_group),
                object_dimensions: mint::Vector2 { x: 1.0, y: 1.0 },
            };
            self.instance_buffer_capacity = new_capacity;
        }

        let mut transform_data = vec![0; transforms.len() * INSTANCE_SIZE];
        for (transform, transform_data_slice) in transforms
            .iter()
            .zip(transform_data.chunks_exact_mut(INSTANCE_SIZE))
        {
            transform_data_slice.copy_from_slice(bytemuck::cast_slice(
                &transform
                    .as_matrix(mint::Vector2 { x: 1.0, y: 1.0 })
                    .to_cols_array_2d(),
            ));
        }
        ctx.queue.write_buffer(
            &self.data.instance_buffer,
            self.instance_buffer_count * INSTANCE_SIZE as u64,
            &transform_data,
        );

        self.instance_buffer_count = new_count;
    }

    /// Pushes this image batch to the draw queue.
    pub fn draw(&self, render: &mut Render, transform: Transform) {
        render.push_data(
            self.data.clone(),
            self.instance_buffer_count as u32,
            transform,
            0,
        );
    }
}
