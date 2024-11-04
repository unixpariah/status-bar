use crate::Wgpu;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle, WaylandWindowHandle};
use std::ptr::NonNull;
use wayland_client::{
    protocol::{wl_output, wl_surface},
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;

pub struct Output {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    wl_surface: wl_surface::WlSurface,
    output: wl_output::WlOutput,
}

impl Output {
    pub fn new(
        output: wl_output::WlOutput,
        wl_surface: wl_surface::WlSurface,
        layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        instance: &wgpu::Instance,
        raw_display_handle: RawDisplayHandle,
    ) -> Self {
        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(wl_surface.id().as_ptr() as *mut _).unwrap(),
        ));

        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle,
                    raw_window_handle,
                })
                .unwrap()
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("Failed to find suitable adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None))
            .expect("Failed to request device");

        Self {
            output,
            adapter,
            device,
            queue,
            surface,
            layer_surface,
            wl_surface,
        }
    }
}

impl Dispatch<zxdg_output_manager_v1::ZxdgOutputManagerV1, ()> for Wgpu {
    fn event(
        _state: &mut Self,
        _proxy: &zxdg_output_manager_v1::ZxdgOutputManagerV1,
        _event: <zxdg_output_manager_v1::ZxdgOutputManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_output::WlOutput, ()> for Wgpu {
    fn event(
        _state: &mut Self,
        _proxy: &wl_output::WlOutput,
        _event: <wl_output::WlOutput as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for Wgpu {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for Wgpu {
    fn event(
        state: &mut Self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let output = state
            .outputs
            .iter()
            .find(|output| output.layer_surface == *layer_surface)
            .unwrap();

        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                output.layer_surface.ack_configure(serial);

                state.width = width;
                state.height = height;

                let adapter = &output.adapter;
                let surface = &output.surface;
                let device = &output.device;
                let queue = &output.queue;

                let cap = surface.get_capabilities(&adapter);
                let surface_config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: cap.formats[0],
                    view_formats: vec![cap.formats[0]],
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    width: state.width,
                    height: state.height,
                    desired_maximum_frame_latency: 2,
                    // Wayland is inherently a mailbox system.
                    present_mode: wgpu::PresentMode::Mailbox,
                };

                surface.configure(&output.device, &surface_config);

                // We don't plan to render much in this example, just clear the surface.
                let surface_texture = surface
                    .get_current_texture()
                    .expect("failed to acquire next swapchain texture");
                let texture_view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&Default::default());
                {
                    let _renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &texture_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                }

                // Submit the command in the queue to execute
                queue.submit(Some(encoder.finish()));
                surface_texture.present();
            }
            _ => {}
        }
    }
}
