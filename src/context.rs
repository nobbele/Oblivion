use std::rc::Rc;

use crate::{
    helpers::{get_adapter_surface, get_device_queue},
    pipelines::{
        mesh_pipeline::{create_mesh_pipeline, MeshPipeline},
        texture_pipeline::{create_texture_pipeline, TexturePipeline},
    },
    vertices::{
        image_vertex::{ImageVertex, IMAGE_INDICES, IMAGE_VERTICES},
        mesh_vertex::MeshVertex,
    },
    MeshBuffer, Render, Vertex,
};

pub struct GraphicsContext {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    surface: wgpu::Surface,
    #[allow(dead_code)]
    pub(crate) config: wgpu::SurfaceConfiguration,

    pub(crate) texture_pipeline: Rc<TexturePipeline>,
    pub(crate) texture_mesh_buffer: Rc<MeshBuffer>,

    pub(crate) mesh_pipeline: Rc<MeshPipeline>,
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

        let texture_pipeline = create_texture_pipeline(
            &device,
            config.format,
            &[ImageVertex::desc()],
            wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/quadtex.wgsl").into()),
        );

        let texture_mesh_buffer = MeshBuffer::from_slices(&device, IMAGE_VERTICES, IMAGE_INDICES);

        let mesh_pipeline = create_mesh_pipeline(
            &device,
            config.format,
            &[MeshVertex::desc()],
            wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/mesh.wgsl").into()),
        );

        GraphicsContext {
            surface,
            device,
            queue,
            config,
            texture_pipeline: Rc::new(texture_pipeline),
            texture_mesh_buffer: Rc::new(texture_mesh_buffer),
            mesh_pipeline: Rc::new(mesh_pipeline),
        }
    }

    // TODO Result
    pub fn submit_render(&self, mut render: Render) {
        let output = self
            .surface
            .get_current_texture()
            .expect("Unable to get surface texture");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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

            for drawable in &render.queue {
                drawable.render(&mut render_pass);
            }
        }
        render.queue.clear();

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
