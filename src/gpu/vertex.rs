use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2d {
    pub pos: [f32; 2],
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
pub const VERTICES: &[Vertex2d] = &[
    Vertex2d { pos: [-0.5,  0.5] },
    Vertex2d { pos: [-0.5, -0.5] },
    Vertex2d { pos: [ 0.5, -0.5] },
    Vertex2d { pos: [ 0.5,  0.5] },
];

#[rustfmt::skip]
// wgpu automatically aligns to 4bytes,
// anything smaller than u32 has no real gain
pub const INDICES: &[u32] = &[
  0, 1, 2,
  0, 2, 3,
];

pub struct VertexBuffer {
    pub vertex: Buffer,
    pub index: Buffer,
}

pub fn init_buffers(device: &Device) -> VertexBuffer {
    let vertex = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&VERTICES[..]),
        usage: BufferUsages::VERTEX,
    });
    let index = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&INDICES[..]),
        usage: BufferUsages::INDEX,
    });
    VertexBuffer { vertex, index }
}
