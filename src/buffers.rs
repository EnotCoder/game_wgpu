use wgpu::*;
use winit::dpi::PhysicalSize;

use crate::constants::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub translation: [f32; 4],
    pub rotation: [f32; 4],
    pub projection: [f32; 16],
    pub use_texture: i32,
    pub _padding0: [f32; 3],
    pub light_dir: [f32; 4],
}

pub struct DepthBuffer {
    pub _texture: Texture,
    pub view: TextureView,
}

impl DepthBuffer {
    pub fn new(device: &Device, size: PhysicalSize<u32>) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        Self {
            _texture: texture,
            view,
        }
    }

    pub fn resize(&mut self, device: &Device, new_size: PhysicalSize<u32>) {
        *self = Self::new(device, new_size);
    }
}

pub fn create_perspective_matrix(aspect: f32, fov: f32, near: f32, far: f32) -> [f32; 16] {
    let f = 1.0 / (fov * 0.5).tan();
    [
        f / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        far / (far - near),
        1.0,
        0.0,
        0.0,
        -far * near / (far - near),
        0.0,
    ]
}

pub struct Buffers {
    pub projection: [f32; 16],
    pub depth_buffer: DepthBuffer,
    pub depth_stencil: DepthStencilState,
    pub bind_group_layout: BindGroupLayout,
    pub texture_bind_group_layout: BindGroupLayout,
}

pub fn init_buffers(window_size: PhysicalSize<u32>, device: &Device) -> Buffers {
    let aspect = window_size.width as f32 / window_size.height as f32;
    let projection = create_perspective_matrix(aspect, CAMERA_FOV, CAMERA_NEAR, CAMERA_FAR);

    let depth_buffer = DepthBuffer::new(device, window_size);

    let depth_stencil = DepthStencilState {
        format: DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: CompareFunction::Less,
        stencil: StencilState::default(),
        bias: DepthBiasState::default(),
    };

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Texture Bind Group Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2,
                    sample_type: TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    Buffers {
        projection,
        depth_buffer,
        depth_stencil,
        bind_group_layout,
        texture_bind_group_layout,
    }
}
