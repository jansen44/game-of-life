use wgpu::{util::DeviceExt, Buffer, BufferUsages, Device};

use crate::cell::{Cell, CellInstance};

pub struct InstanceBuffers {
    pub cells: Buffer,
}

impl InstanceBuffers {
    pub fn cell_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CellInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

pub fn init_cell_instances(
    device: &Device,
    cells: &[Cell],
    scale_factor: f32,
    offset: f32,
) -> InstanceBuffers {
    let instance_data: Vec<CellInstance> = cells
        .iter()
        .map(|c| CellInstance::from_cell(c, scale_factor, offset))
        .collect();
    let cells = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&instance_data),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    });
    InstanceBuffers { cells }
}
