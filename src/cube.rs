use nalgebra_glm::Vec3;
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;

pub struct Cube {
    pub center: Vec3,
    pub dim_x: f32,  // Mitad de la dimensión en x
    pub dim_y: f32,  // Mitad de la dimensión en y
    pub dim_z: f32,  // Mitad de la dimensión en z
    pub material: Material,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let min = self.center - Vec3::new(self.dim_x, self.dim_y, self.dim_z);
        let max = self.center + Vec3::new(self.dim_x, self.dim_y, self.dim_z);

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

        if t_min > tz_max || tz_min > t_max {
            return Intersect::empty();
        }

        if tz_min > t_min {
            t_min = tz_min;
        }

        if tz_max < t_max {
            t_max = tz_max;
        }

        if t_min < 0.0 {
            return Intersect::empty();
        }

        let point = ray_origin + ray_direction * t_min;
        let normal = (point - self.center).normalize();
        let distance = t_min;

        Intersect::new(point, normal, distance, self.material)
    }
}