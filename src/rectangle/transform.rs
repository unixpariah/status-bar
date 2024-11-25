use super::Rectangle;

// Name              | Implemented by Struct | Implemented by Shader
// ------------------|-----------------------|-----------------------
// scale             | [x]                   | [x]
// skew              | [x]                   | [ ]
// rotate            | [x]                   | [ ]

impl Rectangle {
    pub fn set_scale(&mut self, x: f32, y: f32) -> &mut Self {
        self.scale = [x, y];
        self
    }

    pub fn set_skew(&mut self, x: f32, y: f32) -> &mut Self {
        self.skew = [x, y];
        self
    }

    pub fn set_rotate(&mut self, rotation: f32) -> &mut Self {
        self.rotate = rotation;
        self
    }
}
