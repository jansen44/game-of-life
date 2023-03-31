mod instance;
mod pipeline;
mod uniform;
mod vertex;

use crate::{
    cell::{Cell, CellInstance},
    state::{GRID_COLUMN_SIZE, GRID_LINE_SIZE},
};
use vertex::{VertexBuffer, INDICES};
use wgpu::{Device, Queue, Surface, SurfaceCapabilities, SurfaceConfiguration};

use self::{
    instance::{init_cell_instances, InstanceBuffers},
    pipeline::{init_pipeline, Pipeline},
    uniform::init_uniforms,
    vertex::init_buffers,
};

pub struct Gpu {
    device: Device,
    queue: Queue,
    surface: Surface,

    square_pipeline: Pipeline,

    square_buffers: VertexBuffer,
    instance_buffers: InstanceBuffers,

    pub surface_config: SurfaceConfiguration,
    pub surface_caps: SurfaceCapabilities,
}

impl Gpu {
    pub async fn new(window: &winit::window::Window, cells: &[Cell]) -> Self {
        let dimensions = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            width: dimensions.width,
            height: dimensions.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            format: surface_caps.formats[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let square_buffers = init_buffers(&device);
        let instance_buffers = init_cell_instances(&device, cells);
        let uniform_buffers = init_uniforms(&device, dimensions);
        let square_pipeline = init_pipeline(&device, &surface_config, &uniform_buffers);

        Self {
            device,
            surface,
            queue,
            surface_caps,
            surface_config,

            square_pipeline,
            square_buffers,
            instance_buffers,
        }
    }

    pub fn resize(&mut self, dimensions: winit::dpi::PhysicalSize<u32>) {
        self.surface_config.width = dimensions.width;
        self.surface_config.height = dimensions.height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn update_cells(&self, cells: &[Cell]) {
        let instance_data: Vec<CellInstance> =
            cells.iter().map(|c| CellInstance::from(c)).collect();
        self.queue.write_buffer(
            &self.instance_buffers.cells,
            0,
            bytemuck::cast_slice(&instance_data),
        )
    }

    pub fn render(&self) {
        let output = self.surface.get_current_texture().unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.02,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.square_pipeline.pipeline);
            pass.set_bind_group(0, &self.square_pipeline.bindgroups.projection_mat, &[]);

            pass.set_vertex_buffer(0, self.square_buffers.vertex.slice(..));
            pass.set_index_buffer(
                self.square_buffers.index.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            pass.set_vertex_buffer(1, self.instance_buffers.cells.slice(..));

            pass.draw_indexed(
                0..INDICES.len() as u32,
                0,
                0..(GRID_COLUMN_SIZE * GRID_LINE_SIZE) as _,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
