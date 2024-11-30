pub mod config;
pub mod wgpu_surface;

use crate::{buffers, rectangle::Rectangle, StatusBar};
use raw_window_handle::RawDisplayHandle;
use wayland_client::{protocol::wl_surface, Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{self, Anchor};

use super::tree;

pub struct Surface {
    pub wgpu: wgpu_surface::WgpuSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub surface: wl_surface::WlSurface,
    pub config: config::Config,
    pub background: tree::Tree,
}

impl Surface {
    pub fn new(
        layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        surface: wl_surface::WlSurface,
        raw_display_handle: RawDisplayHandle,
        instance: &wgpu::Instance,
    ) -> Self {
        let mut config = config::Config::default();
        config.position = config::Position::Left;
        config.background_color = [0.0, 0.0, 0.0, 0.0];

        layer_surface.set_size(1, 1);
        layer_surface.set_anchor(Anchor::Top);
        surface.commit();

        let mut surface = Self {
            wgpu: wgpu_surface::WgpuSurface::new(&surface, raw_display_handle, instance),
            layer_surface,
            surface,
            config,
            background: tree::Tree::new(Rectangle::default()),
        };

        surface.apply_config();

        surface.background.add_child(
            Rectangle::default()
                .set_background_color(0.0, 0.0, 1.0, 1.0)
                .set_size(100.0, 100.0)
                .set_coordinates(100.0, 700.0)
                .set_border_radius(0.0, 10.0, 30.0, 50.0)
                .set_border_color(1.0, 1.0, 1.0, 1.0)
                .set_border_size(2.0, 2.0, 2.0, 2.0),
        );

        surface.background.add_child(
            Rectangle::default()
                .set_background_color(1.0, 0.0, 0.0, 1.0)
                .set_size(300.0, 300.0)
                .set_coordinates(200.0, 100.0)
                .set_border_radius(10.0, 10.0, 10.0, 10.0),
        );

        surface.background.add_child(
            Rectangle::default()
                .set_background_color(0.0, 1.0, 0.0, 1.0)
                .set_size(100.0, 100.0)
                .set_coordinates(10.0, 100.0)
                .set_border_radius(55.0, 55.0, 55.0, 55.0)
                .set_boxshadow_offset(0.0, 10.0)
                .set_boxshadow_color(1.0, 1.0, 0.0, 1.0)
                .set_boxshadow_softness(30.0),
        );

        surface.background.add_child(
            Rectangle::default()
                .set_background_color(0.0, 1.0, 0.0, 1.0)
                .set_size(100.0, 100.0)
                .set_coordinates(100.0, 500.0)
                .set_border_radius(10.0, 10.0, 10.0, 10.0)
                .set_border_size(0.0, 5.0, 10.0, 15.0)
                .set_border_color(1.0, 1.0, 0.0, 1.0)
                .set_outline_width(5.0)
                .set_outline_color(1.0, 0.0, 0.0, 1.0)
                .set_outline_offset(50.0)
                .set_boxshadow_offset(0.0, 10.0)
                .set_boxshadow_color(1.0, 1.0, 0.0, 1.0)
                .set_boxshadow_softness(30.0),
        );

        surface.background.add_child(
            Rectangle::default()
                .set_background_color(0.0, 1.0, 0.0, 1.0)
                .set_size(100.0, 100.0)
                .set_coordinates(100.0, 500.0)
                .set_border_radius(10.0, 10.0, 10.0, 10.0)
                .set_border_size(0.0, 5.0, 10.0, 15.0)
                .set_border_color(1.0, 1.0, 0.0, 1.0)
                .set_outline_width(5.0)
                .set_outline_color(1.0, 0.0, 0.0, 1.0)
                .set_outline_offset(50.0)
                .set_boxshadow_offset(0.0, 10.0)
                .set_boxshadow_color(1.0, 1.0, 0.0, 1.0)
                .set_boxshadow_softness(30.0),
        );

        surface
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let background = std::mem::take(&mut self.background.data);
        self.background.data = background.set_size(width as f32, height as f32);

        self.wgpu.resize(width, height);
        self.wgpu.projection_uniform = buffers::ProjectionUniform::new(
            &self.wgpu.device,
            0.0,
            width as f32,
            0.0,
            height as f32,
        );
    }

    pub fn apply_config(&mut self) {
        let anchor = match self.config.position {
            config::Position::Top => Anchor::Top | Anchor::Left | Anchor::Right,
            config::Position::Bottom => Anchor::Bottom | Anchor::Left | Anchor::Right,
            config::Position::Left => Anchor::Top | Anchor::Left | Anchor::Bottom,
            config::Position::Right => Anchor::Top | Anchor::Right | Anchor::Bottom,
        };

        self.layer_surface.set_anchor(anchor);
        self.layer_surface
            .set_exclusive_zone(self.config.size as i32);
        self.layer_surface.set_layer(self.config.layer);
        self.layer_surface.set_margin(
            self.config.margin.top as i32,
            self.config.margin.right as i32,
            self.config.margin.bottom as i32,
            self.config.margin.left as i32,
        );

        let color = self.config.background_color;
        let background = std::mem::take(&mut self.background.data);
        self.background.data =
            background.set_background_color(color[0], color[1], color[2], color[3]);
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for StatusBar {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for StatusBar {
    fn event(
        state: &mut Self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
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
        output.surface.resize(width, height);
    }
}
