mod surface;

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
        render_pass.set_bind_group(0, &self.surface.wgpu.projection_uniform.bind_group, &[]);

        let rect_buf = buffers::VertexBuffer::new(
            &self.surface.wgpu.device,
            &[
                buffers::Vertex {
                    position: [0.0, 1.0],
                },
                buffers::Vertex {
                    position: [1.0, 1.0],
                },
                buffers::Vertex {
                    position: [1.0, 0.0],
                },
                buffers::Vertex {
                    position: [0.0, 0.0],
                },
            ],
        );
        render_pass.set_vertex_buffer(0, rect_buf.slice(..));

        let instance = self.surface.rectangle.get_instance();
        let instance_two = Rectangle::default()
            .set_background_color(0.0, 0.0, 1.0, 1.0)
            .set_size(100.0, 100.0)
            .set_coordinates(100.0, 700.0)
            .set_border_radius(0.0, 10.0, 30.0, 50.0)
            .set_border_color(1.0, 1.0, 1.0, 1.0)
            .set_border_size(2.0, 2.0, 2.0, 2.0)
            .get_instance();

        let instance_three = Rectangle::default()
            .set_background_color(1.0, 0.0, 0.0, 1.0)
            .set_size(300.0, 300.0)
            .set_coordinates(200.0, 100.0)
            .set_border_radius(10.0, 10.0, 10.0, 10.0)
            .get_instance();

        let instance_four = Rectangle::default()
            .set_background_color(0.0, 1.0, 0.0, 1.0)
            .set_size(100.0, 100.0)
            .set_coordinates(10.0, 100.0)
            .set_border_radius(55.0, 55.0, 55.0, 55.0)
            .get_instance();

        let instance_five = Rectangle::default()
            .set_background_color(0.0, 1.0, 0.0, 1.0)
            .set_size(100.0, 100.0)
            .set_coordinates(100.0, 500.0)
            .set_border_radius(10.0, 10.0, 10.0, 10.0)
            .set_border_size(0.0, 5.0, 10.0, 15.0)
            .set_border_color(1.0, 1.0, 1.0, 1.0)
            .set_outline_width(5.0)
            .set_outline_color(1.0, 0.0, 0.0, 1.0)
            .set_outline_offset(50.0)
            .get_instance();

        let instances: Vec<_> = vec![
            instance,
            instance_two,
            instance_three,
            instance_four,
            instance_five,
        ];
        let instance_buffer = buffers::InstanceBuffer::new(&self.surface.wgpu.device, &instances);
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

        render_pass.set_index_buffer(
            self.surface.wgpu.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(
            0..self.surface.wgpu.index_buffer.size(),
            0,
            0..instance_buffer.size(),
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
            zxdg_output_v1::Event::Name { name } => output.info.name = name,
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
