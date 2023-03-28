use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};
use winit::dpi::PhysicalSize;

use crate::math::ortho_projection;

pub struct UniformBuffers {
    pub projection_mat: Buffer,
}

pub fn init_uniforms(device: &Device, dimensions: PhysicalSize<u32>) -> UniformBuffers {
    let dimensions = (dimensions.width, dimensions.height);
    let projection_mat = ortho_projection(dimensions);

    let projection_mat = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&projection_mat),
        usage: BufferUsages::UNIFORM,
    });

    UniformBuffers { projection_mat }
}
