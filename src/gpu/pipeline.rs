use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, Device, FragmentState, MultisampleState, PipelineLayoutDescriptor,
    PrimitiveState, PrimitiveTopology, RenderPipeline, ShaderStages, SurfaceConfiguration,
    VertexState,
};

use super::{instance::InstanceBuffers, uniform::UniformBuffers, vertex::Vertex2d};

pub struct BindGroups {
    pub projection_mat: BindGroup,
    pub projection_mat_layout: BindGroupLayout,
}

pub fn init_bind_groups(device: &Device, buffers: &UniformBuffers) -> BindGroups {
    let projection_mat_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            count: None,
            visibility: ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }],
    });
    let projection_mat = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &projection_mat_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: buffers.projection_mat.as_entire_binding(),
        }],
    });
    BindGroups {
        projection_mat,
        projection_mat_layout,
    }
}

pub struct Pipeline {
    pub pipeline: RenderPipeline,
    pub bindgroups: BindGroups,
}

pub fn init_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    buffers: &UniformBuffers,
) -> Pipeline {
    let square_shader = device.create_shader_module(include_wgsl!("shaders/square.wgsl"));
    let bindgroups = init_bind_groups(device, buffers);

    let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bindgroups.projection_mat_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        depth_stencil: None,
        multiview: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },

        layout: Some(&layout),
        vertex: VertexState {
            module: &square_shader,
            buffers: &[Vertex2d::desc(), InstanceBuffers::cell_desc()],
            entry_point: "vs_main",
        },
        fragment: Some(FragmentState {
            entry_point: "fs_main",
            module: &square_shader,
            targets: &[Some(wgpu::ColorTargetState {
                blend: Some(wgpu::BlendState::REPLACE),
                format: config.format,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
    });

    Pipeline {
        pipeline,
        bindgroups,
    }
}
