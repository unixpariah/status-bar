use crate::buffers;

pub struct Rectangle {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [f32; 4],
}

impl Rectangle {
    pub fn set_position(&mut self, x: u32, y: u32) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn set_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn set_color(&mut self, color: [f32; 4]) -> &mut Self {
        self.color = color;
        self
    }

    pub fn set_vertex_buffer(&self, device: &wgpu::Device, render_pass: &mut wgpu::RenderPass) {
        let vertices: &[buffers::Vertex] = &[
            buffers::Vertex {
                position: [self.x as f32, (self.y + self.height) as f32],
                color: self.color,
            },
            buffers::Vertex {
                position: [(self.x + self.width) as f32, (self.y + self.height) as f32],
                color: self.color,
            },
            buffers::Vertex {
                position: [(self.x + self.width) as f32, self.y as f32],
                color: self.color,
            },
            buffers::Vertex {
                position: [self.x as f32, self.y as f32],
                color: self.color,
            },
        ];
        let vertex_buffer = buffers::VertexBuffer::new(device, vertices);

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            color: [0.0, 0.0, 0.0, 0.0],
        }
    }
}
