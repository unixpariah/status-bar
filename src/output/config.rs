use crate::buffers;

use super::wgpu_surface;
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

pub struct Config {
    pub size: u32,
    margin: Margin,
    position: Position,
    layer: zwlr_layer_shell_v1::Layer,
    pub background_color: [f64; 4],
}

impl Config {
    pub fn apply(
        &self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        width: u32,
        height: u32,
        wgpu_surface: &mut wgpu_surface::WgpuSurface,
    ) {
        let (width, height) = match self.position {
            Position::Bottom | Position::Top => (width, self.size),
            Position::Right | Position::Left => (self.size, height),
        };

        let (anchor, exclusive_zone, size) = match self.position {
            Position::Top => (
                Anchor::Top | Anchor::Left | Anchor::Right,
                height as i32,
                (
                    width as u32 - self.margin.left - self.margin.right,
                    self.size,
                ),
            ),
            Position::Bottom => (
                Anchor::Bottom | Anchor::Left | Anchor::Right,
                height as i32,
                (
                    width as u32 - self.margin.left - self.margin.right,
                    self.size,
                ),
            ),
            Position::Left => (
                Anchor::Top | Anchor::Left | Anchor::Bottom,
                width as i32,
                (
                    self.size,
                    height as u32 - self.margin.top - self.margin.bottom,
                ),
            ),
            Position::Right => (
                Anchor::Top | Anchor::Right | Anchor::Bottom,
                width as i32,
                (
                    self.size,
                    height as u32 - self.margin.top - self.margin.bottom,
                ),
            ),
        };

        layer_surface.set_anchor(anchor);
        layer_surface.set_exclusive_zone(exclusive_zone);
        layer_surface.set_size(size.0, size.1);

        layer_surface.set_layer(self.layer);

        layer_surface.set_margin(
            self.margin.top as i32,
            self.margin.right as i32,
            self.margin.bottom as i32,
            self.margin.left as i32,
        );

        wgpu_surface.resize(width, height);

        let cap = wgpu_surface.surface.get_capabilities(&wgpu_surface.adapter);
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

        wgpu_surface
            .surface
            .configure(&wgpu_surface.device, &surface_config);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            size: 500,
            margin: Margin::default(),
            position: Position::Right,
            layer: Layer::Top,
            background_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}
