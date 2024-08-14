use crate::color::Color;
use crate::bmp;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<Color>,
    pub background_color: Color,
    pub current_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let background_color = Color::new(0, 0, 0); 
        let current_color = Color::new(0, 0, 0);    
        let buffer_size = (width * height) as usize;
        Self {
            width,
            height,
            buffer: vec![background_color; buffer_size],
            background_color,
            current_color,
        }
    }

    pub fn set_background_color(&mut self, hex: u32) {
        self.background_color = Color::from_hex(hex);
    }

    pub fn set_current_color(&mut self, hex: u32) {
        self.current_color = Color::from_hex(hex);
    }

    pub fn clear(&mut self) {
        self.buffer.fill(self.background_color);
    }

    pub fn point(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.buffer[index] = self.current_color;
        }
    }

    pub fn render(&self, file_path: &str) -> std::io::Result<()> {
        bmp::write_bmp_file(file_path, &self.buffer, self.width, self.height)
    }

    pub fn get_u32_buffer(&self) -> Vec<u32> {
        self.buffer.iter().map(|color| {
            ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
        }).collect()
    }
}