use std::ops::{Deref, DerefMut};

use crate::{buffers, rectangle};

pub struct Tree {
    node: Node,
}

impl Tree {
    pub fn new(rectangle: rectangle::Rectangle) -> Self {
        Self {
            node: Node::new(rectangle),
        }
    }

    pub fn add_child(&mut self, rectangle: rectangle::Rectangle) {
        let node = Node {
            data: rectangle,
            children: Vec::new(),
        };

        self.children.push(node);
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        render_pass: &mut wgpu::RenderPass,
        index_buffer: &buffers::IndexBuffer,
    ) {
        let rect_buf = buffers::VertexBuffer::new(
            &device,
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

        let mut indices = self
            .children
            .iter()
            .map(|node| node.data.get_instance())
            .collect::<Vec<buffers::Instance>>();

        indices.push(self.data.get_instance());

        let instance_buffer = buffers::InstanceBuffer::new(device, &indices);
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..index_buffer.size(), 0, 0..instance_buffer.size());
    }
}

impl Deref for Tree {
    type Target = Node;
    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl DerefMut for Tree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

pub struct Node {
    pub children: Vec<Node>,
    pub data: rectangle::Rectangle,
}

impl Node {
    pub fn new(rectangle: rectangle::Rectangle) -> Self {
        return Self {
            data: rectangle,
            children: Vec::new(),
        };
    }
}
