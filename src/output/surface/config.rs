use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::{self, Layer};

pub enum Position {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Default)]
pub struct Margin {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

pub struct Config {
    pub size: u32,
    pub margin: Margin,
    pub position: Position,
    pub layer: zwlr_layer_shell_v1::Layer,
    pub background_color: [f32; 4],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            size: 500,
            margin: Margin::default(),
            position: Position::Right,
            layer: Layer::Top,
            background_color: [0.65, 0.89, 0.63, 1.0],
        }
    }
}
