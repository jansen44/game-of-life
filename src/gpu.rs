use wgpu::{
    util::DeviceExt, BindGroup, Device, PipelineLayoutDescriptor, Queue, RenderPipeline,
    RenderPipelineDescriptor, Surface, SurfaceCapabilities, SurfaceConfiguration,
};
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex2d {
    pos: [f32; 2],
}

impl Vertex2d {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

#[rustfmt::skip]
const VERTICES: &[Vertex2d] = &[
    Vertex2d { pos: [-1.0,  1.0] },
    Vertex2d { pos: [-1.0, -1.0] },
    Vertex2d { pos: [ 1.0, -1.0] },
    Vertex2d { pos: [ 1.0,  1.0] },
];

#[rustfmt::skip]
// wgpu automatically aligns to 4bytes,
// anything smaller than u32 has no real gain
const INDICES: &[u32] = &[
  0, 1, 2,
  0, 2, 3,
];

fn normalized_vertices(dimensions: PhysicalSize<u32>) -> Vec<Vertex2d> {
    let vertices = VERTICES.to_owned();
    let aspect_ratio = dimensions.width as f32 / dimensions.height as f32;

    vertices
        .into_iter()
        .map(|e| Vertex2d {
            pos: [e.pos[0], e.pos[1] * aspect_ratio],
        })
        .collect()
}

pub struct Gpu {
    device: Device,
    queue: Queue,
    surface: Surface,
    square_pipeline: RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    _uni_buffer: wgpu::Buffer,
    uni_bind_group: BindGroup,

    pub surface_config: SurfaceConfiguration,
    pub surface_caps: SurfaceCapabilities,
}

impl Gpu {
    pub async fn new(window: &winit::window::Window) -> Self {
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

        let (vertex_buffer, index_buffer, uni_buffer) = Self::init_buffers(&device, dimensions);
        let (square_pipeline, uni_bind_group) =
            Self::init_pipelines(&device, &surface_config, &uni_buffer);

        Self {
            device,
            surface,
            queue,
            surface_caps,
            surface_config,
            square_pipeline,
            vertex_buffer,
            index_buffer,
            _uni_buffer: uni_buffer,
            uni_bind_group,
        }
    }

    fn init_buffers(
        device: &Device,
        dimensions: PhysicalSize<u32>,
    ) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
        let vertex = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&normalized_vertices(dimensions)[..]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indice = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        #[rustfmt::skip]
        let transform_uni: [f32; 16] = [
            0.01, 0.0, 0.0, 0.0,
            0.0, 0.01, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];

        let uni = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&transform_uni),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        (vertex, indice, uni)
    }

    fn init_pipelines(
        device: &Device,
        config: &SurfaceConfiguration,
        uni_buffer: &wgpu::Buffer,
    ) -> (RenderPipeline, BindGroup) {
        let square_shader = device.create_shader_module(wgpu::include_wgsl!("square.wgsl"));

        let uni_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    ty: wgpu::BufferBindingType::Uniform,
                },
            }],
        });

        let uni_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &uni_bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uni_buffer.as_entire_binding(),
            }],
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uni_bg_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Square Pipeline"),
            depth_stencil: None,
            multiview: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },

            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &square_shader,
                buffers: &[Vertex2d::desc()],
                entry_point: "vs_main",
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: "fs_main",
                module: &square_shader,
                targets: &[Some(wgpu::ColorTargetState {
                    blend: Some(wgpu::BlendState::REPLACE),
                    format: config.format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
        });

        (pipeline, uni_bg)
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

            pass.set_pipeline(&self.square_pipeline);
            pass.set_bind_group(0, &self.uni_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
