use std::{
    ops::{Deref, RangeBounds},
    rc::Rc,
};
use wgpu::{util::DeviceExt, BufferAddress};

use crate::math::{self, Projection};

pub struct Buffer<T> {
    buffer: wgpu::Buffer,
    vertices: Rc<[T]>,
}

impl<T> Buffer<T> {
    fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, data: &[T]) -> Self
    where
        T: bytemuck::Pod,
    {
        Self {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Buffer"),
                usage,
                contents: bytemuck::cast_slice(data),
            }),
            vertices: data.into(),
        }
    }

    pub fn size(&self) -> u32 {
        self.vertices.len() as u32
    }

    pub fn slice<S>(&self, bounds: S) -> wgpu::BufferSlice<'_>
    where
        S: RangeBounds<BufferAddress>,
    {
        self.buffer.slice(bounds)
    }
}

impl<T> Deref for Buffer<T> {
    type Target = wgpu::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct VertexBuffer(Buffer<Vertex>);

impl VertexBuffer {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex]) -> Self {
        VertexBuffer(Buffer::new(
            device,
            wgpu::BufferUsages::VERTEX,
            vertices.into(),
        ))
    }
}

impl Deref for VertexBuffer {
    type Target = Buffer<Vertex>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct IndexBuffer(Buffer<u16>);

impl IndexBuffer {
    pub fn new(device: &wgpu::Device, indices: &[u16]) -> Self {
        IndexBuffer(Buffer::new(device, wgpu::BufferUsages::INDEX, indices))
    }
}

impl Deref for IndexBuffer {
    type Target = Buffer<u16>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub enum Uniform {
    Projection(ProjectionUniform),
}

pub struct ProjectionUniform {
    pub buffer: wgpu::Buffer,
    pub projection: math::Mat4,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl ProjectionUniform {
    pub fn new(device: &wgpu::Device, left: f32, right: f32, top: f32, bottom: f32) -> Self {
        let projection = math::Mat4::projection(left, right, top, bottom);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Buffer"),
            usage: wgpu::BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(&projection),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("bind group layout"),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            buffer,
            projection,
            bind_group,
            bind_group_layout,
        }
    }
}

// You get the correct color using the following formula: rgb_color = ((srgb_color / 255 + 0.055) / 1.055) ^ 2.4