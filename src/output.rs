use crate::Wgpu;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle, WaylandWindowHandle};
use std::ptr::NonNull;
use wayland_client::{
    protocol::{wl_output, wl_surface},
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_v1;
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer},
    zwlr_layer_surface_v1::{self, Anchor},
};

enum Position {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Default)]
struct Margin {
    left: u32,
    right: u32,
    top: u32,
    bottom: u32,
}

struct Config {
    size: u32,
    margin: Margin,
    position: Position,
    layer: zwlr_layer_shell_v1::Layer,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            size: 50,
            margin: Margin::default(),
            position: Position::Top,
            layer: Layer::Top,
        }
    }
}

impl Config {
    fn apply(
        &self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        width: u32,
        height: u32,
    ) {
        let width = match self.position {
            Position::Bottom | Position::Top => width,
            Position::Right | Position::Left => self.size,
        };

        let height = match self.position {
            Position::Bottom | Position::Top => self.size,
            Position::Right | Position::Left => height,
        };

        match self.position {
            Position::Top => {
                layer_surface.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right);
                layer_surface.set_exclusive_zone(height as i32);

                layer_surface.set_size(
                    width as u32 - self.margin.left - self.margin.right,
                    self.size,
                );
            }
            Position::Left => {
                layer_surface.set_anchor(Anchor::Top | Anchor::Left | Anchor::Bottom);
                layer_surface.set_exclusive_zone(width as i32);

                layer_surface.set_size(
                    self.size,
                    height as u32 - self.margin.top - self.margin.bottom,
                );
            }
            Position::Right => {
                layer_surface.set_anchor(Anchor::Top | Anchor::Right | Anchor::Bottom);
                layer_surface.set_exclusive_zone(width as i32);

                layer_surface.set_size(
                    self.size,
                    height as u32 - self.margin.top - self.margin.bottom,
                );
            }
            Position::Bottom => {
                layer_surface.set_anchor(Anchor::Bottom | Anchor::Left | Anchor::Right);
                layer_surface.set_exclusive_zone(height as i32);

                layer_surface.set_size(
                    width as u32 - self.margin.left - self.margin.right,
                    self.size,
                );
            }
        }

        layer_surface.set_layer(self.layer);

        layer_surface.set_margin(
            self.margin.top as i32,
            self.margin.right as i32,
            self.margin.bottom as i32,
            self.margin.left as i32,
        );
    }
}

pub struct Output {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    wl_surface: wl_surface::WlSurface,
    output: wl_output::WlOutput,
    xdg_output: zxdg_output_v1::ZxdgOutputV1,
    pub info: OutputInfo,
    config: Config,
}

impl Output {
    pub fn new(
        output: wl_output::WlOutput,
        xdg_output: zxdg_output_v1::ZxdgOutputV1,
        wl_surface: wl_surface::WlSurface,
        layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        id: u32,
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
            config: Config::default(),
            xdg_output,
            output,
            adapter,
            device,
            queue,
            surface,
            layer_surface,
            wl_surface,
            info: OutputInfo::new(id),
        }
    }
}

pub struct OutputInfo {
    name: String,
    width: i32,
    height: i32,
    scale: i32,
    pub id: u32,
}

impl OutputInfo {
    fn new(id: u32) -> Self {
        Self {
            name: "".to_string(),
            width: 0,
            height: 0,
            scale: 1,
            id,
        }
    }
}

impl Dispatch<zxdg_output_v1::ZxdgOutputV1, ()> for Wgpu {
    fn event(
        state: &mut Self,
        xdg_output: &zxdg_output_v1::ZxdgOutputV1,
        event: zxdg_output_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let output = state
            .outputs
            .iter_mut()
            .find(|output| output.xdg_output == *xdg_output)
            .unwrap();

        match event {
            zxdg_output_v1::Event::Name { name } => output.info.name = name,
            zxdg_output_v1::Event::LogicalSize { width, height } => {
                output.info.width = width;
                output.info.height = height;
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for Wgpu {
    fn event(
        state: &mut Self,
        wl_output: &wl_output::WlOutput,
        event: wl_output::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let output = state
            .outputs
            .iter_mut()
            .find(|output| output.output == *wl_output)
            .unwrap();

        match event {
            wl_output::Event::Scale { factor } => {
                output.info.scale = factor;
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for Wgpu {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: wl_surface::Event,
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
            .iter_mut()
            .find(|output| output.layer_surface == *layer_surface)
            .unwrap();

        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                output.layer_surface.ack_configure(serial);

                output.config.apply(&layer_surface, width, height);

                let cap = output.surface.get_capabilities(&output.adapter);
                let surface_config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: cap.formats[0],
                    view_formats: vec![cap.formats[0]],
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    width,
                    height,
                    desired_maximum_frame_latency: 2,
                    present_mode: wgpu::PresentMode::Mailbox,
                };

                output.surface.configure(&output.device, &surface_config);

                // We don't plan to render much in this example, just clear the surface.
                let surface_texture = output
                    .surface
                    .get_current_texture()
                    .expect("failed to acquire next swapchain texture");
                let texture_view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = output.device.create_command_encoder(&Default::default());
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
                output.queue.submit(Some(encoder.finish()));
                surface_texture.present();
            }
            _ => {}
        }
    }
}
