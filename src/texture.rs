extern crate image;

use image::{DynamicImage, GenericImageView, Pixel, RgbaImage};

#[derive(Debug, Clone)]

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: RgbaImage,  // Almacena la imagen en formato RGBA
}

impl Texture {
    // Cargar la textura desde un archivo
    pub fn from_file(path: &str) -> Result<Self, String> {
        match image::open(path) {
            Ok(img) => {
                let rgba_img = img.to_rgba8();  // Convierte la imagen a RGBA
                let (width, height) = rgba_img.dimensions();

                Ok(Texture {
                    width,
                    height,
                    data: rgba_img,
                })
            }
            Err(e) => Err(format!("Error loading texture: {}", e)),
        }
    }

    // Obtener el color de un píxel (como un array de 4 valores RGBA)
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let pixel = self.data.get_pixel(x, y);
        let channels = pixel.channels();  // Obtén los canales como una referencia al array
        [channels[0], channels[1], channels[2], channels[3]]  // Convierte la referencia en un array
    }
    
}
