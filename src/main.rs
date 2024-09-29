
mod framebuffer;
mod ray_intersect;
mod cube;
mod color;
mod camera;
mod light;
mod material;
mod texture;

use minifb::{ Window, WindowOptions, Key };
use nalgebra_glm::{Vec3, normalize};
use std::time::Duration;
use std::f32::consts::PI;
use std::rc::Rc;

use crate::color::Color;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;
use crate::texture::Texture;

const ORIGIN_BIAS: f32 = 1e-4;
const SKYBOX_COLOR: Color = Color::new(68, 142, 228);

fn offset_origin(intersect: &Intersect, direction: &Vec3) -> Vec3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if direction.dot(&intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
    let cosi = -incident.dot(normal).max(-1.0).min(1.0);
    
    let (n_cosi, eta, n_normal);

    if cosi < 0.0 {
        n_cosi = -cosi;
        eta = 1.0 / eta_t;
        n_normal = -normal;
    } else {
        n_cosi = cosi;
        eta = eta_t;
        n_normal = *normal;
    }
    
    let k = 1.0 - eta * eta * (1.0 - n_cosi * n_cosi);
    
    if k < 0.0 {
        reflect(incident, &n_normal)
    } else {
        eta * incident + (eta * n_cosi - k.sqrt()) * n_normal
    }
}

fn fresnel(incident: &Vec3, normal: &Vec3, ior: f32) -> f32 {
    let mut cosi = incident.dot(normal).clamp(-1.0, 1.0);
    let etai = 1.0;
    let etat = ior;
    let sint = etai / etat * (1.0 - cosi * cosi).sqrt();

    if sint >= 1.0 {
        return 1.0;
    } else {
        let cost = (1.0 - sint * sint).sqrt();
        cosi = cosi.abs();
        let rs = ((etat * cosi) - (etai * cost)) / ((etat * cosi) + (etai * cost));
        let rp = ((etai * cosi) - (etat * cost)) / ((etai * cosi) + (etat * cost));
        return (rs * rs + rp * rp) / 2.0;
    }
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Cube],
    camera: &Camera
) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let shadow_ray_origin = offset_origin(intersect, &light_dir);
    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir, &camera.eye);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
            break;
        }
    }

    shadow_intensity
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Cube],
    lights: &[Light],
    depth: u32,
    camera: &Camera,
) -> Color {
    if depth > 3 {
        return SKYBOX_COLOR;
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction,&camera.eye);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return SKYBOX_COLOR;
    }

    let mut final_color = Color::black();

    for light in lights {
        let light_dir = (light.position - intersect.point).normalize();
        let view_dir = (ray_origin - intersect.point).normalize();
        let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

        let shadow_intensity = cast_shadow(&intersect, light, objects, camera);
        let light_intensity = light.intensity * (1.0 - shadow_intensity);

        let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
        let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

        let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
        let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;
        let kr = fresnel(ray_direction, &intersect.normal, intersect.material.refractive_index,);

        let mut reflect_color = Color::green();
        let reflectivity = kr * intersect.material.albedo[2];
        if reflectivity > 0.0 {
            let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
            let reflect_origin = offset_origin(&intersect, &reflect_dir);
            reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, lights, depth + 1, camera);
        }

        let mut refract_color = Color::green();
        let transparency = (1.0 - kr) * intersect.material.albedo[3];
        if transparency > 0.0 {
            let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
            let refract_origin = offset_origin(&intersect, &refract_dir);
            refract_color = cast_ray(&refract_origin, &refract_dir, objects, lights, depth + 1, camera);
        }

        final_color = final_color + (diffuse + specular) * (1.0 - reflectivity - transparency)
            + (reflect_color * reflectivity)
            + (refract_color * transparency);
    }

    final_color
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, lights: &[Light]) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

            let rotated_direction = camera.base_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, lights, 0, camera);

            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}


fn main() {
    let window_width = 400;
    let window_height = 300;
    let framebuffer_width = 400;
    let framebuffer_height = 300;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "Refractor",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    
    let grass_textures: [Option<Texture>; 6] = [
        Texture::from_file("src/textures/grass_top.png").ok(),
        Texture::from_file("src/textures/grass_top.png").ok(),
        Texture::from_file("src/textures/grass_top.png").ok(),
        Texture::from_file("src/textures/grass_top.png").ok(),
        Texture::from_file("src/textures/grass_top.png").ok(),
        Texture::from_file("src/textures/grass_top.png").ok(),
    ];

    let water_textures: [Option<Texture>; 6] = [
        Texture::from_file("src/textures/water.png").ok(),
        Texture::from_file("src/textures/water.png").ok(),
        Texture::from_file("src/textures/water.png").ok(),
        Texture::from_file("src/textures/water.png").ok(),
        Texture::from_file("src/textures/water.png").ok(),
        Texture::from_file("src/textures/water.png").ok(),
    ];

    let wood_textures: [Option<Texture>; 6] = [
        Texture::from_file("src/textures/wood.png").ok(),
        Texture::from_file("src/textures/wood.png").ok(),
        Texture::from_file("src/textures/wood.png").ok(),
        Texture::from_file("src/textures/wood.png").ok(),
        Texture::from_file("src/textures/wood.png").ok(),
        Texture::from_file("src/textures/wood.png").ok(),
    ];

    let furnace_textures: [Option<Texture>; 6] = [
        Texture::from_file("src/textures/furnace_front.png").ok(),
        Texture::from_file("src/textures/wood.png").ok(),
        Texture::from_file("src/textures/wood.png").ok(),
        Texture::from_file("src/textures/wood.png").ok(),
        Texture::from_file("src/textures/wood.png").ok(),
        Texture::from_file("src/textures/wood.png").ok(),

    ];


    let grass = Rc::new(
        Material::new(        
            Color::new(96, 160, 54),
            50.0,
            [1.0, 0.0, 0.0, 0.0],
            0.0,
            grass_textures,
            None,
    ));

    let water = Rc::new(
        Material::new(
            Color::new(10, 40, 225),
            50.0,
            [1.0, 0.1, 0.0, 0.0],
            0.0,
            water_textures,
            None,
        )
    );
    
    let wood = Rc::new(
        Material::new(    
            Color::new(10, 40, 225),
        50.0,
        [0.1, 0.1, 0.0, 0.0],
        0.0,
        wood_textures,
        None,
    ));

    let furnace = Rc::new(
        Material::new(        
        Color::new(10, 40, 225),
        50.0,
        [0.1, 0.1, 0.0, 0.0],
        0.0,
        furnace_textures,
        None,
    ));
    

    let cube_size = 2.75;

    let objects = [
        //River 2*3
        Cube {center: Vec3::new(0.0 , -0.6, cube_size * -8.0), dim_x: cube_size * 2.0, dim_y: cube_size - 0.6, dim_z: cube_size * 3.0, material: Rc::clone(&water),},
        //Lake 7*6
        Cube {center: Vec3::new(cube_size * 1.0 , -0.6, cube_size * 1.0), dim_x: cube_size * 7.0, dim_y: cube_size - 0.6, dim_z: cube_size * 6.0, material: Rc::clone(&water),},


        // Floor 4*3
        Cube {center: Vec3::new(cube_size * -6.0, 0.0, -cube_size * 8.0), dim_x: cube_size * 4.0, dim_y: cube_size, dim_z: cube_size * 3.0, material: Rc::clone(&grass),},
        //Floor 2*6
        Cube {center: Vec3::new(cube_size * -8.0, 0.0, -cube_size * -1.0), dim_x: cube_size * 2.0, dim_y: cube_size, dim_z: cube_size * 6.0, material:  Rc::clone(&grass),},
        //Floor 10*1
        Cube {center: Vec3::new(cube_size * 0.0, 0.0, -cube_size * -8.0), dim_x: cube_size * 10.0, dim_y: cube_size, dim_z: cube_size * 1.0, material:  Rc::clone(&grass),},
        //Floor 1*6
        Cube {center: Vec3::new(cube_size * 9.0, 0.0, -cube_size * -1.0), dim_x: cube_size * 1.0, dim_y: cube_size, dim_z: cube_size * 6.0, material:  Rc::clone(&grass),},
        //Floor 4*3
        Cube {center: Vec3::new(cube_size * 6.0, 0.0, -cube_size * 8.0), dim_x: cube_size * 4.0, dim_y: cube_size, dim_z: cube_size * 3.0, material:  Rc::clone(&grass),},
        

        //Objetos
        //Mesa de Crafteo 1*1
        Cube {center: Vec3::new(cube_size *  -9.0, cube_size * 2.0, -cube_size * -6.0), dim_x: cube_size, dim_y: cube_size, dim_z: cube_size, material:  Rc::clone(&wood),},
        //Horno 1*1
        Cube {center: Vec3::new(cube_size *  -9.0, cube_size * 2.0, -cube_size * -4.0), dim_x: cube_size, dim_y: cube_size, dim_z: cube_size, material:  Rc::clone(&furnace),},
        //Tronco 1*1*4
        Cube {center: Vec3::new(cube_size *  -7.0, cube_size * 4.0, -cube_size * 8.0), dim_x: cube_size, dim_y: cube_size * 4.0, dim_z: cube_size, material:  Rc::clone(&wood),},

    ];

    

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 100.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let sun = Light::new(
        Vec3::new(0.0, 40.0, 0.0),
        Color::new(255, 255, 224),
        2.0,
    );
    
    let moon = Light::new(
        Vec3::new(0.0, -40.0, 0.0),
        Color::new(173, 216, 230),
        0.5,
    );
    // Centro de la órbita
    let center = Vec3::new(0.0, 0.0, 0.0);
    let radius = 40.0;
    let mut angle_sun = 0.0;
    let mut angle_moon = std::f32::consts::PI;
    
    // Simulación de tiempo
    let delta_time = 0.2;
    let mut lights: [Light; 2] = [sun, moon];
    let rotation_speed = PI/10.0;


    while window.is_open() && !window.is_key_down(Key::Escape) {

        // Actualiza las posiciones del Sol y la Luna
        lights[0].update_position_orbit(center, radius, angle_sun); // Actualiza el Sol
        lights[1].update_position_orbit(center, radius, angle_moon); // Actualiza la Luna
        
        // Incrementa los ángulos para orbitar
        angle_sun += delta_time;
        angle_moon += delta_time;
        
        // Para que los ángulos no desborden
        if angle_sun > 2.0 * std::f32::consts::PI {
            angle_sun -= 2.0 * std::f32::consts::PI;
        }
        if angle_moon > 2.0 * std::f32::consts::PI {
            angle_moon -= 2.0 * std::f32::consts::PI;
        }

        lights[0].light_condition();



        render(&mut framebuffer, &objects, &camera, &lights);

        if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {
            camera.orbit(rotation_speed, 0.0); 
        }

        if window.is_key_down(Key::Right)||
        window.is_key_down(Key::D) {
            camera.orbit(-rotation_speed, 0.0);
        }

        if window.is_key_down(Key::Up)||
        window.is_key_down(Key::W) {
            camera.orbit(0.0, -rotation_speed);
        }

        if window.is_key_down(Key::Down)||
        window.is_key_down(Key::S) {
            camera.orbit(0.0, rotation_speed);
        }


        render(&mut framebuffer, &objects, &camera, &lights);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}   
