mod border;
mod filter;
mod image;
mod outline;
mod transform;

use crate::buffers;

pub enum BoxSizing {
    ContentBox,
    BorderBox,
}

#[derive(Default)]
struct BoxShadow {
    x_offset: f32,
    y_offset: f32,
    blur_radius: f32,
    spread_radius: f32,
    color: [f32; 4],
    inset: bool,
}

#[derive(Default)]
pub struct PaddingSize {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl PaddingSize {
    pub fn to_array(&self) -> [f32; 4] {
        [self.top, self.right, self.bottom, self.left]
    }
}

// Feature Parity
// Name             | Implemented by Struct | Implemented by Shader
// -----------------|-----------------------|-----------------------
// x                 | [x]                   | [x]
// y                 | [x]                   | [x]
// width             | [x]                   | [x]
// height            | [x]                   | [x]
// bg-color          | [x]                   | [x]
// bg-image          | [ ]                   | [ ]
// box-sizing        | [x]                   | [x]
// padding           | [x]                   | [x]
// border            | [x]                   | [x]
// box-shadow        | [x]                   | [ ]
// outline           | [x]                   | [x]
pub struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    background_color: [f32; 4],
    padding: PaddingSize,
    box_sizing: BoxSizing,
    border: border::Border,
    outline: outline::Outline,
    box_shadow: BoxShadow,
    blur: f32,
    brightness: f32,
    contrast: f32,
    grayscale: f32,
    hue_rotate: f32,
    rotate: f32,
    invert: f32,
    saturate: f32,
    sepia: f32,
}

pub struct Extents {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn set_coordinates(&mut self, x: f32, y: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn set_size(&mut self, width: f32, height: f32) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn set_box_sizing(&mut self, box_sizing: BoxSizing) -> &mut Self {
        self.box_sizing = box_sizing;
        self
    }

    pub fn set_padding(&mut self, top: f32, right: f32, bottom: f32, left: f32) -> &mut Self {
        self.padding = PaddingSize {
            top,
            right,
            bottom,
            left,
        };
        self
    }

    pub fn set_background_color(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.background_color = [r * a, g * a, b * a, a];
        self
    }

    // Getter for extents
    pub fn get_extents(&self) -> Extents {
        let (width, height) = match self.box_sizing {
            BoxSizing::ContentBox => (
                self.width
                    + self.padding.left
                    + self.padding.right
                    + self.border.size.left
                    + self.border.size.right,
                self.height
                    + self.padding.top
                    + self.padding.bottom
                    + self.border.size.top
                    + self.border.size.bottom,
            ),
            BoxSizing::BorderBox => (self.width, self.height),
        };

        Extents {
            x: self.x,
            y: self.y,
            width,
            height,
        }
    }

    pub fn get_instance(&self) -> buffers::Instance {
        let extents = self.get_extents();

        // Extents only cover parts of rectangle that interact with the rest of ui
        let x = extents.x - self.outline.width - self.outline.offset;
        let y = extents.y - self.outline.width - self.outline.offset;

        let width = extents.width + (self.outline.width + self.outline.offset) * 2.0;
        let height = extents.height + (self.outline.width + self.outline.offset) * 2.0;

        buffers::Instance {
            position: [x, y],
            size: [width, height],
            color: self.background_color,
            border_radius: self.border.radius.to_array(),
            border_size: self.border.size.to_array(),
            border_color: self.border.color,
            outline_width: self.outline.width,
            outline_offset: self.outline.offset,
            outline_color: self.outline.color,
        }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            blur: 0.0,
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            padding: PaddingSize::default(),
            background_color: [0.0, 0.0, 0.0, 0.0],
            border: border::Border::default(),
            outline: outline::Outline::default(),
            box_sizing: BoxSizing::ContentBox,
            box_shadow: BoxShadow::default(),
            brightness: 1.0,
            contrast: 1.0,
            grayscale: 0.0,
            hue_rotate: 0.0,
            rotate: 0.0,
            invert: 0.0,
            saturate: 1.0,
            sepia: 0.0,
        }
    }
}
