extern crate image;

use image::{DynamicImage, GenericImageView, ImageReader};

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub color_array: Vec<Vec<u32>>,
}

impl Texture {
    pub fn new(file_path: &str) -> Texture {
        let img = ImageReader::open(file_path).expect("Failed to open image file").decode().expect("Failed to decode image");
        let (width, height) = img.dimensions();
        
        let color_array = Self::load_array(&img, width, height);
        
        Texture { width, height, color_array }
    }

    fn load_array(img: &DynamicImage, width: u32, height: u32) -> Vec<Vec<u32>> {
        let mut color_array = vec![vec![0; height as usize]; width as usize];

        for (x, y, pixel) in img.pixels() {
            let rgb = pixel.to_rgb();
            let color = ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32);
            color_array[x as usize][y as usize] = color;
        }

        color_array
    }

    pub fn get_pixel_color(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.color_array[x as usize][y as usize]
        } else {
            0xFF0000 
        }
    }
}
