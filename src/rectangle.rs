use crate::buffers;

pub enum BoxSizing {
    ContentBox,
    BorderBox,
}

pub enum OutlineStyle {
    None,
    Solid,
    Dotted,
    Dashed,
    Double,
    Groove,
    Ridge,
    Hidden,
}

struct Outline {
    width: f32,
    color: [f32; 4],
    style: OutlineStyle,
}

pub enum BorderStyle {
    None,
    Solid,
    Dotted,
    Dashed,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
    Hidden,
}

struct Border {
    radius: [f32; 4],
    size: BoxSpacing,
    color: [f32; 4],
    style: BorderStyle,
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
struct BoxSpacing {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

impl BoxSpacing {
    fn to_array(&self) -> [f32; 4] {
        [self.top, self.right, self.bottom, self.left]
    }
}

pub struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    background_color: [f32; 4],
    padding: BoxSpacing,
    box_sizing: BoxSizing,
    border: Border,
    outline: Outline,
    box_shadow: BoxShadow,
    blur: f32,
    brightness: f32,
    contrast: f32,
    grayscale: f32,
    hue_rotate: f32,
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
    // Setters with chaining
    pub fn set_outline_width(&mut self, width: f32) -> &mut Self {
        self.outline.width = width;
        self
    }

    pub fn set_outline_color(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.outline.color = [r, g, b, a];
        self
    }

    pub fn set_outline_style(&mut self, style: OutlineStyle) -> &mut Self {
        self.outline.style = style;
        self
    }

    pub fn set_coordinates(&mut self, x: f32, y: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn set_blur(&mut self, blur: f32) -> &mut Self {
        self.blur = blur;
        self
    }

    pub fn set_brightness(&mut self, brightness: f32) -> &mut Self {
        self.brightness = brightness;
        self
    }

    pub fn set_contrast(&mut self, contrast: f32) -> &mut Self {
        self.contrast = contrast;
        self
    }

    pub fn set_grayscale(&mut self, grayscale: f32) -> &mut Self {
        self.grayscale = grayscale;
        self
    }

    pub fn set_hue_rotate(&mut self, hue_rotate: f32) -> &mut Self {
        self.hue_rotate = hue_rotate;
        self
    }

    pub fn set_invert(&mut self, invert: f32) -> &mut Self {
        self.invert = invert;
        self
    }

    pub fn set_saturate(&mut self, saturate: f32) -> &mut Self {
        self.saturate = saturate;
        self
    }

    pub fn set_sepia(&mut self, sepia: f32) -> &mut Self {
        self.sepia = sepia;
        self
    }

    pub fn set_size(&mut self, width: f32, height: f32) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn set_opacity(&mut self, opacity: f32) -> &mut Self {
        self.background_color[3] = opacity;
        self
    }

    pub fn set_box_sizing(&mut self, box_sizing: BoxSizing) -> &mut Self {
        self.box_sizing = box_sizing;
        self
    }

    pub fn set_border_size(&mut self, top: f32, right: f32, bottom: f32, left: f32) -> &mut Self {
        self.border.size = BoxSpacing {
            top,
            right,
            bottom,
            left,
        };
        self
    }

    pub fn set_border_color(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.border.color = [r, g, b, a];
        self
    }

    pub fn set_border_style(&mut self, style: BorderStyle) -> &mut Self {
        self.border.style = style;
        self
    }

    pub fn set_border_radius(
        &mut self,
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    ) -> &mut Self {
        self.border.radius = [top_left, top_right, bottom_right, bottom_left];
        self
    }

    pub fn set_padding(&mut self, top: f32, right: f32, bottom: f32, left: f32) -> &mut Self {
        self.padding = BoxSpacing {
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

    pub fn get_vertices(&self) -> [buffers::Vertex; 4] {
        let extents = self.get_extents();

        [
            buffers::Vertex {
                position: [extents.x, extents.y + extents.height],
            },
            buffers::Vertex {
                position: [extents.x + extents.width, extents.y + extents.height],
            },
            buffers::Vertex {
                position: [extents.x + extents.width, extents.y],
            },
            buffers::Vertex {
                position: [extents.x, extents.y],
            },
        ]
    }

    pub fn get_instance(&self) -> buffers::Instance {
        let extents = self.get_extents();
        let width = extents.width;
        let height = extents.height;

        buffers::Instance {
            position: [extents.x, extents.y],
            size: [width, height],
            color: self.background_color,
            border_radius: self.border.radius,
            border_size: self.border.size.to_array(),
            border_color: self.border.color,
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
            padding: BoxSpacing::default(),
            background_color: [0.0, 0.0, 0.0, 0.0],
            border: Border {
                radius: [0.0, 0.0, 0.0, 0.0],
                color: [0.0, 0.0, 0.0, 0.0],
                size: BoxSpacing::default(),
                style: BorderStyle::Solid,
            },
            outline: Outline {
                color: [0.0, 0.0, 0.0, 0.0],
                width: 0.0,
                style: OutlineStyle::Solid,
            },
            box_sizing: BoxSizing::ContentBox,
            box_shadow: BoxShadow::default(),
            brightness: 1.0,
            contrast: 1.0,
            grayscale: 0.0,
            hue_rotate: 0.0,
            invert: 0.0,
            saturate: 1.0,
            sepia: 0.0,
        }
    }
}
