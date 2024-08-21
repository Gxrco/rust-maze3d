use rusttype::{Font, Scale, point, PositionedGlyph};

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height],
            background_color: 0x000000,
            current_color: 0xFFFFFF,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.fill(self.background_color);
    }

    pub fn point(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = self.current_color;
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    pub fn drawtext(&mut self, text: &str, x: usize, y: usize, scale: Scale, color: u32) {
        let font = Self::load_font();
        let v_metrics = font.v_metrics(scale);
        let offset = point(x as f32, y as f32 + v_metrics.ascent);

        let glyphs = font.layout(text, scale, offset);

        self.draw_glyphs(glyphs, color);
    }

    pub fn text_width(&self, text: &str, scale: Scale) -> f32 {
        let font = Self::load_font();
        font.layout(text, scale, point(0.0, 0.0))
            .map(|g| g.pixel_bounding_box().map(|b| b.width() as f32).unwrap_or(0.0))
            .sum()
    }

    fn load_font() -> Font<'static> {
        static FONT_DATA: &'static [u8] = include_bytes!("../assets/Sterion.ttf");
        Font::try_from_bytes(FONT_DATA).expect("Error loading font")
    }

    fn draw_glyphs<'a>(&mut self, glyphs: impl Iterator<Item = PositionedGlyph<'a>>, color: u32) {
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, gv| {
                    if gv > 0.5 {
                        let gx = gx as i32 + bounding_box.min.x;
                        let gy = gy as i32 + bounding_box.min.y;

                        if let (Ok(gx), Ok(gy)) = (usize::try_from(gx), usize::try_from(gy)) {
                            if gx < self.width && gy < self.height {
                                self.buffer[gy * self.width + gx] = color;
                            }
                        }
                    }
                });
            }
        }
    }
}
