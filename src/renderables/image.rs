use std::rc::Rc;

use wgpu::util::DeviceExt;

use crate::{
    pipelines::texture_pipeline::TexturePipeline, GraphicsContext, MeshBuffer, Render, Renderable,
};

pub struct Image {
    imp: Rc<ImageImpl>,
}

impl Image {
    // TODO Result
    pub fn new(ctx: &GraphicsContext, width: u32, height: u32, data: &[u8]) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
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
            layout: &ctx.texture_pipeline.bind_group_layout,
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
            label: Some("Oblivion_TextureBindGroup"),
        });

        Image {
            imp: Rc::new(ImageImpl {
                pipeline: Rc::clone(&ctx.texture_pipeline),
                mesh_buffer: Rc::clone(&ctx.texture_mesh_buffer),
                bind_group,
                //texture,
                //texture_view,
                //sampler,
            }),
        }
    }

    pub fn draw(&self, render: &mut Render) {
        render.queue.push(Rc::clone(&self.imp) as Rc<_>)
    }
}

pub(crate) struct ImageImpl {
    pipeline: Rc<TexturePipeline>,
    mesh_buffer: Rc<MeshBuffer>,
    bind_group: wgpu::BindGroup,
    //texture: wgpu::Texture,
    //texture_view: wgpu::TextureView,
    //sampler: wgpu::Sampler,
}

impl Renderable for ImageImpl {
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.mesh_buffer.vertex.0.slice(..));
        render_pass.set_index_buffer(
            self.mesh_buffer.index.0.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..self.mesh_buffer.index.1, 0, 0..1);
    }
}
