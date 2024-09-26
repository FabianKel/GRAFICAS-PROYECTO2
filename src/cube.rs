use nalgebra_glm::Vec3;
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;
use std::rc::Rc;

pub struct Cube {
    pub center: Vec3,
    pub dim_x: f32,  // Mitad de la dimensión en x
    pub dim_y: f32,  // Mitad de la dimensión en y
    pub dim_z: f32,  // Mitad de la dimensión en z
    pub material: Rc<Material>,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3, camera_position: &Vec3) -> Intersect {
        // Calcular las esquinas mínimas y máximas del cubo
        let min = self.center - Vec3::new(self.dim_x, self.dim_y, self.dim_z);
        let max = self.center + Vec3::new(self.dim_x, self.dim_y, self.dim_z);

        // Calcular tmin y tmax para cada eje (x, y, z)
        let mut t_min = (min.x - ray_origin.x) / ray_direction.x;
        let mut t_max = (max.x - ray_origin.x) / ray_direction.x;

        if t_min > t_max {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        let mut ty_min = (min.y - ray_origin.y) / ray_direction.y;
        let mut ty_max = (max.y - ray_origin.y) / ray_direction.y;

        if ty_min > ty_max {
            std::mem::swap(&mut ty_min, &mut ty_max);
        }

        // Verificar si hay intersección en el eje y
        if t_min > ty_max || ty_min > t_max {
            return Intersect::empty();
        }

        if ty_min > t_min {
            t_min = ty_min;
        }
        if ty_max < t_max {
            t_max = ty_max;
        }

        let mut tz_min = (min.z - ray_origin.z) / ray_direction.z;
        let mut tz_max = (max.z - ray_origin.z) / ray_direction.z;

        if tz_min > tz_max {
            std::mem::swap(&mut tz_min, &mut tz_max);
        }

        // Verificar si hay intersección en el eje z
        if t_min > tz_max || tz_min > t_max {
            return Intersect::empty();
        }

        if tz_min > t_min {
            t_min = tz_min;
        }
        if tz_max < t_max {
            t_max = tz_max;
        }

        // Si t_min es negativo, la intersección está detrás del origen del rayo
        if t_min < 0.0 {
            return Intersect::empty();
        }

       // Calcular el punto de intersección y la normal de la cara
       let intersection_point = ray_origin + ray_direction * t_min;
       let face_normal = self.calculate_normal(&intersection_point, &min, &max);

       // Verificar si la cara es visible desde la cámara
       if !self.is_face_visible(&face_normal, camera_position, &intersection_point) {
           return Intersect::empty();
       }

       // Si la cara es visible, continuar con el cálculo
       let distance = t_min;
       Intersect::new(intersection_point, face_normal, distance, Rc::clone(&self.material))
   }
}

impl Cube {

    pub fn is_face_visible(&self, face_normal: &Vec3, camera_position: &Vec3, intersection_point: &Vec3) -> bool {
        // Dirección desde el punto de intersección hacia la cámara
        let view_direction = (camera_position - intersection_point).normalize();
        // Producto punto entre la normal de la cara y la dirección de la cámara
        let dot_product = face_normal.dot(&view_direction);

        // Si el producto punto es positivo, la cara está orientada hacia la cámara
        dot_product > 0.0
    }

    // Función para calcular la normal en función del punto de intersección
    fn calculate_normal(&self, point: &Vec3, min: &Vec3, max: &Vec3) -> Vec3 {
        let epsilon = 1e-4;
        
        if (point.x - min.x).abs() < epsilon {
            Vec3::new(-1.0, 0.0, 0.0)
        } else if (point.x - max.x).abs() < epsilon {
            Vec3::new(1.0, 0.0, 0.0)
        } else if (point.y - min.y).abs() < epsilon {
            Vec3::new(0.0, -1.0, 0.0)
        } else if (point.y - max.y).abs() < epsilon {
            Vec3::new(0.0, 1.0, 0.0)
        } else if (point.z - min.z).abs() < epsilon {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }

    fn get_texture_coordinates(&self, point: &Vec3, min: &Vec3, max: &Vec3) -> (f32, f32) {
        let epsilon = 1e-4;

        if (point.x - min.x).abs() < epsilon {
            let u = (point.z - min.z) / (max.z - min.z);
            let v = (point.y - min.y) / (max.y - min.y);
            (u, v)
        } else if (point.x - max.x).abs() < epsilon {
            let u = (point.z - min.z) / (max.z - min.z);
            let v = (max.y - point.y) / (max.y - min.y);
            (u, v)
        } else if (point.y - min.y).abs() < epsilon {
            let u = (point.x - min.x) / (max.x - min.x);
            let v = (point.z - min.z) / (max.z - min.z);
            (u, v)
        } else if (point.y - max.y).abs() < epsilon {
            let u = (point.x - min.x) / (max.x - min.x);
            let v = (max.z - point.z) / (max.z - min.z);
            (u, v)
        } else if (point.z - min.z).abs() < epsilon {
            let u = (point.x - min.x) / (max.x - min.x);
            let v = (point.y - min.y) / (max.y - min.y);
            (u, v)
        } else {
            let u = (point.x - min.x) / (max.x - min.x);
            let v = (max.y - point.y) / (max.y - min.y);
            (u, v)
        }
    }
}
