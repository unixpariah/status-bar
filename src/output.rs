mod surface;
mod tree;

use crate::{buffers, rectangle::Rectangle, StatusBar};
use raw_window_handle::RawDisplayHandle;
use surface::config;
use wayland_client::{
    protocol::{wl_output, wl_surface},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_v1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;

pub struct OutputInfo {
    name: Option<String>,
    width: i32,
    height: i32,
    scale: i32,
    pub id: u32,
}

impl OutputInfo {
    fn new(id: u32) -> Self {
        Self {
            name: None,
            width: 0,
            height: 0,
            scale: 1,
            id,
        }
    }
}

pub struct Output {
    surface: surface::Surface,
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
        raw_display_handle: RawDisplayHandle,
        instance: &wgpu::Instance,
    ) -> Self {
        let surface = surface::Surface::new(layer_surface, surface, raw_display_handle, instance);

        Self {
            xdg_output,
            output,
            info: OutputInfo::new(id),
            surface,
        }
    }

    pub fn render(&mut self) {
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
        render_pass.set_bind_group(0, &self.surface.wgpu.projection_uniform.bind_group, &[]);

        self.surface.background.render(
            &self.surface.wgpu.device,
            &mut render_pass,
            &self.surface.wgpu.index_buffer,
        );

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
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let output = state
            .outputs
            .iter_mut()
            .find(|output| output.xdg_output == *xdg_output)
            .unwrap(); // Can't be called if this xdg_output wasn't created

        match event {
            zxdg_output_v1::Event::Name { name } => output.info.name = Some(name),
            zxdg_output_v1::Event::LogicalSize { width, height } => {
                output.info.width = width;
                output.info.height = height;

                let (width, height) = match output.surface.config.position {
                    config::Position::Top => (width as u32, output.surface.config.size),
                    config::Position::Bottom => (width as u32, output.surface.config.size),
                    config::Position::Left => (output.surface.config.size, height as u32),
                    config::Position::Right => (output.surface.config.size, height as u32),
                };

                output
                    .surface
                    .layer_surface
                    .set_size(width as u32, height as u32);
                output.surface.surface.commit();
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
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
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
