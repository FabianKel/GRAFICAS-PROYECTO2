// src/texture.rs

use image::GenericImageView;
use crate::Color; // Asumo que tienes una estructura Color

// Función para cargar la textura desde una imagen
pub fn load_texture() -> Vec<Color> {
    let img = image::open("src/textures/grass1.webp").expect("Failed to load texture");
    let mut texture: Vec<Color> = Vec::new();

    for (_, _, pixel) in img.pixels() {
        let r = pixel[0] as u8;
        let g = pixel[1] as u8;
        let b = pixel[2] as u8;
        texture.push(Color::new(r, g, b));
    }

    texture
}

// Función para aplicar la textura al floor
pub fn apply_texture(texture: &[Color], u: f32, v: f32) -> Color {
    let width = (texture.len() as f32).sqrt() as usize; // Asumimos que la textura es cuadrada
    let x = (u * (width as f32)).clamp(0.0, width as f32 - 1.0) as usize;
    let y = (v * (width as f32)).clamp(0.0, width as f32 - 1.0) as usize;

    texture[x + y * width]
}


