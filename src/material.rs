use crate::color::Color;
use crate::texture::Texture;

#[derive(Debug, Clone)]
pub struct Material {
  pub diffuse: Color,
  pub specular: f32,
  pub albedo: Vec<f32>,  // Este es un vector
  pub refractive_index: f32,
  pub textures: [Option<Texture>; 6],  // Este es un array de texturas
  pub normal_map: Option<Texture>,
}


impl Material {
    pub fn new(
        diffuse: Color,
        specular: f32,
        albedo: [f32; 4],
        refractive_index: f32,
        textures: [Option<Texture>; 6],   // Recibe un array de 6 texturas opcionales
        normal_map: Option<Texture>,      // Mapa de normales opcional
    ) -> Self {
        Material {
            diffuse,
            specular,
            albedo: albedo.to_vec(),
            refractive_index,
            textures,
            normal_map,
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Color::new(0, 0, 0),
            specular: 0.0,
            albedo: vec![0.0, 0.0, 0.0, 0.0],
            refractive_index: 0.0,
            textures: [None, None, None, None, None, None],  // Sin texturas por defecto
            normal_map: None,
        }
    }
}
