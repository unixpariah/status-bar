use crate::buffers;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle, WaylandWindowHandle};
use std::ptr::NonNull;
use wayland_client::{protocol::wl_surface, Proxy};

pub struct WgpuSurface {
    pub surface: wgpu::Surface<'static>,
    pub render_pipeline: wgpu::RenderPipeline,
    config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    shader: wgpu::ShaderModule,
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    pub index_buffer: buffers::IndexBuffer,
    pub projection_uniform: buffers::ProjectionUniform,
}

impl WgpuSurface {
    pub fn new(
        surface: &wl_surface::WlSurface,
        raw_display_handle: RawDisplayHandle,
        instance: &wgpu::Instance,
    ) -> Self {
        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(surface.id().as_ptr() as *mut _).unwrap(),
        ));

        let wgpu_surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle,
                    raw_window_handle,
                })
                .unwrap()
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&wgpu_surface),
            ..Default::default()
        }))
        .expect("Failed to find suitable adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None))
            .expect("Failed to request device");

        let projection_uniform = buffers::ProjectionUniform::new(&device, 0.0, 0.0, 0.0, 0.0);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&projection_uniform.bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("../../shader.wgsl"));

        let surface_caps = wgpu_surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .unwrap_or(&surface_caps.formats[0]);

        let alpha_mode = surface_caps
            .alpha_modes
            .iter()
            .find(|a| **a == wgpu::CompositeAlphaMode::PreMultiplied)
            .unwrap_or(&surface_caps.alpha_modes[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface_format,
            width: 1,
            height: 1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: *alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[buffers::Vertex::desc(), buffers::Instance::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: None,
            multiview: None,
            cache: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let indices: &[u16] = &[0, 1, 3, 1, 2, 3];
        let index_buffer = buffers::IndexBuffer::new(&device, indices);

        Self {
            surface: wgpu_surface,
            config,
            render_pipeline,
            adapter,
            shader,
            queue,
            device,
            index_buffer,
            projection_uniform,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }
}
