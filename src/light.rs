
use nalgebra_glm::Vec3;
use crate::color::Color;
use std::fmt;


pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
        }
    }

    pub fn light_condition(&mut self){
        let y_position = self.position.y;
        let x_position = self.position.x;
        if y_position >= 20.0 && y_position < 40.0 {
            println!("Dia");
        } else if y_position >= 0.0 && y_position < 20.0 {
            if x_position >0.0{
                println!("Amanecer");
            }else{
                println!("Atardecer");
            }
        } else {
            println!("Noche");
        }
    }

    pub fn update_position_orbit(&mut self, center: Vec3, radius: f32, angle: f32) {
        let fixed_z = self.position.z;
    
        // Ciclo de x e y para hacer simular el ciclo  de día y noche
        self.position.x = center.x + radius * angle.cos();
        self.position.y = center.y + radius * angle.sin();
        
        self.position.z = fixed_z;
    }
    

    
}



impl fmt::Display for Light {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Light(Position: ({}, {}, {}))",
            self.position.x, self.position.y, self.position.z,
        )
    }
}
