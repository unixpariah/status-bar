mod config;
pub mod wgpu_surface;

use crate::{buffers, rectangle::Rectangle, StatusBar};
use raw_window_handle::RawDisplayHandle;
use wayland_client::{protocol::wl_surface, Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{self, Anchor};

pub struct Surface {
    pub wgpu: wgpu_surface::WgpuSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub surface: wl_surface::WlSurface,
    pub config: config::Config,
    pub rectangle: Rectangle,
}

impl Surface {
    pub fn new(
        layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        surface: wl_surface::WlSurface,
        raw_display_handle: RawDisplayHandle,
        instance: &wgpu::Instance,
    ) -> Self {
        let config = config::Config::default();

        Self {
            wgpu: wgpu_surface::WgpuSurface::new(&surface, raw_display_handle, instance),
            layer_surface,
            surface,
            config,
            rectangle: Rectangle::default(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let (width, height) = (
            width - self.config.margin.right - self.config.margin.left,
            height - self.config.margin.top - self.config.margin.bottom,
        );

        self.wgpu.resize(width, height);
        self.layer_surface.set_size(width, height);
        self.wgpu.projection_uniform = buffers::ProjectionUniform::new(
            &self.wgpu.device,
            self.config.margin.left as f32,
            width as f32,
            self.config.margin.top as f32,
            height as f32,
        );

        self.rectangle.set_position(0, 0).set_size(width, height);
    }

    pub fn apply_config(&mut self, width: u32, height: u32) {
        let (anchor, exclusive_zone) = match self.config.position {
            config::Position::Top => (Anchor::Top | Anchor::Left | Anchor::Right, height as i32),
            config::Position::Bottom => {
                (Anchor::Bottom | Anchor::Left | Anchor::Right, height as i32)
            }
            config::Position::Left => (Anchor::Top | Anchor::Left | Anchor::Bottom, width as i32),
            config::Position::Right => (Anchor::Top | Anchor::Right | Anchor::Bottom, width as i32),
        };

        self.layer_surface.set_anchor(anchor);
        self.layer_surface.set_exclusive_zone(exclusive_zone);
        self.layer_surface.set_layer(self.config.layer);
        self.layer_surface.set_margin(
            self.config.margin.top as i32,
            self.config.margin.right as i32,
            self.config.margin.bottom as i32,
            self.config.margin.left as i32,
        );

        self.rectangle.set_color(self.config.background_color);

        let (width, height) = match self.config.position {
            config::Position::Bottom | config::Position::Top => (width, self.config.size),
            config::Position::Right | config::Position::Left => (self.config.size, height),
        };
        self.resize(width, height);
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
        output.surface.apply_config(width, height);
    }
}
