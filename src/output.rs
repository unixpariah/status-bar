mod config;
pub mod wgpu_surface;

use crate::{buffers, StatusBar};
use wayland_client::{
    protocol::{wl_output, wl_surface},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_v1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;
use wgpu::util::RenderEncoder;

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

pub struct Surface {
    wgpu: wgpu_surface::WgpuSurface,
    layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    surface: wl_surface::WlSurface,
    config: config::Config,
}

pub struct Output {
    surface: Surface,
    output: wl_output::WlOutput,
    xdg_output: zxdg_output_v1::ZxdgOutputV1,
    pub info: OutputInfo,
}

impl Output {
    pub fn new(
        output: wl_output::WlOutput,
        xdg_output: zxdg_output_v1::ZxdgOutputV1,
        surface: wl_surface::WlSurface,
        layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        id: u32,
        wgpu: wgpu_surface::WgpuSurface,
    ) -> Self {
        let config = config::Config::default();

        let surface = Surface {
            wgpu,
            layer_surface,
            surface,
            config,
        };

        Self {
            xdg_output,
            output,
            info: OutputInfo::new(id),
            surface,
        }
    }

    pub fn render(&self) {
        let surface_texture = self
            .surface
            .wgpu
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .surface
            .wgpu
            .device
            .create_command_encoder(&Default::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.surface.wgpu.render_pipeline);
        render_pass.set_bind_group(0, &self.surface.wgpu.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.surface.wgpu.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.surface.wgpu.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..self.surface.wgpu.index_buffer.size(), 0, 0..1);
        drop(render_pass); // Drop renderpass and release mutable borrow on encoder

        self.surface.wgpu.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}

impl Dispatch<zxdg_output_v1::ZxdgOutputV1, ()> for StatusBar {
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
            .unwrap(); // Can't be called if this xdg_output wasn't created

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

impl Dispatch<wl_output::WlOutput, ()> for StatusBar {
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
            .unwrap(); // Can't be called if this wl_output wasn't created

        match event {
            wl_output::Event::Scale { factor } => {
                output.info.scale = factor;
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for StatusBar {
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

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for StatusBar {
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
            .find(|output| output.surface.layer_surface == *layer_surface)
            .unwrap(); // Can't be called if this layer_surface wasn't created

        let zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        else {
            return;
        };

        output.surface.layer_surface.ack_configure(serial);

        output
            .surface
            .config
            .apply(&layer_surface, width, height, &mut output.surface.wgpu);
    }
}
