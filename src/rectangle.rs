use std::rc::Rc;

use crate::buffers;

struct Border {
    radius: [f32; 4],
}

pub struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    background_color: [f32; 4],
    padding: [f32; 4],
    border: Border,
}

impl Rectangle {
    pub fn set_position(&mut self, x: u32, y: u32) -> &mut Self {
        self.x = x as f32;
        self.y = y as f32;
        self
    }

    pub fn set_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.width = width as f32;
        self.height = height as f32;
        self
    }

    pub fn set_padding(&mut self, top: f32, right: f32, bottom: f32, left: f32) -> &mut Self {
        self.padding = [top, right, bottom, left];
        self
    }

    pub fn set_color(&mut self, color: [f32; 4]) -> &mut Self {
        self.background_color = color;
        self.background_color
            .iter_mut()
            .enumerate()
            .for_each(|(i, channel)| {
                if i < 3 {
                    *channel *= color[3]
                }
            });

        self
    }

    pub fn set_radius(
        &mut self,
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    ) -> &mut Self {
        self.border.radius = [top_left, top_right, bottom_right, bottom_left];
        self
    }

    pub fn get_vertices(&self) -> Rc<[buffers::Vertex]> {
        let width = self.x + self.width + self.padding[3] + self.padding[1];
        let height = self.y + self.height + self.padding[0] + self.padding[2];

        Rc::new([
            buffers::Vertex {
                position: [self.x, height],
            },
            buffers::Vertex {
                position: [width, height],
            },
            buffers::Vertex {
                position: [width, self.y],
            },
            buffers::Vertex {
                position: [self.x, self.y],
            },
        ])
    }

    pub fn get_instance(&self) -> buffers::Instance {
        let width = self.width + self.padding[3] + self.padding[1];
        let height = self.height + self.padding[0] + self.padding[2];

        buffers::Instance {
            position: [self.x, self.y],
            size: [width, height],
            color: self.background_color,
            border_radius: self.border.radius,
        }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            padding: [0.0, 0.0, 0.0, 0.0],
            background_color: [0.0, 0.0, 0.0, 0.0],
            border: Border {
                radius: [0.0, 0.0, 0.0, 0.0],
            },
        }
    }
}
