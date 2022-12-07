use std::sync::Arc;

use clap::Parser;
use rand::Rng;
use rapid_qoi::{Colors, Qoi};
use ray::Ray;
use rayon::prelude::*;

use crate::{
    camera::Camera,
    color::Color,
    geometry::{Geometry, Sphere, World},
    material::{Lambertian, Material, Metal},
    vector::Vector,
};

mod camera;
mod color;
mod geometry;
mod material;
mod ray;
mod vector;

#[derive(Parser)]
struct Options {
    width: u32,
    height: u32,
}

/*
fn ray_color(ray: Ray, geometry: &impl Geometry, max_bounces: u32) -> Color {
    if max_bounces == 0 {
        return Color::from_rgb(0.0, 0.0, 0.0);
    }

    match geometry.hit(ray, 0.0001..f64::INFINITY) {
        Some(hit) => match hit.material.scatter(&hit) {
            Some((ray, attenuation)) => attenuation * ray_color(ray, geometry, max_bounces - 1),
            None => Color::from_rgb(0.0, 0.0, 0.0),
        },
        None => {
            let unit_vel = ray.velocity.normalize();

            let t = 0.5 * (unit_vel.y + 1.0f64);
            let top = Color::from_rgb(1.0, 1.0, 1.0);
            let bottom = Color::from_rgb(0.5, 0.7, 1.0);
            top.lerp(bottom, t)
        }
    }
}
*/

fn ray_color_non_recursive(ray: Ray, geometry: &impl Geometry, max_bounces: u32) -> Color {
    let mut color = Color::from_rgb(1.0, 1.0, 1.0);

    let mut curr_ray = ray;

    for _ in 0..max_bounces {
        match geometry.hit(curr_ray, 0.0001..f64::INFINITY) {
            Some(hit) => match hit.material.scatter(&hit) {
                Some((ray, attenuation)) => {
                    color *= attenuation;
                    curr_ray = ray;
                }
                None => return Color::from_rgb(0.0, 0.0, 0.0),
            },
            None => {
                let unit_vel = curr_ray.velocity.normalize_unchecked();

                let t = 0.5 * (unit_vel.y() + 1.0f64);
                let top = Color::from_rgb(1.0, 1.0, 1.0);
                let bottom = Color::from_rgb(0.5, 0.7, 1.0);
                return color * top.lerp(bottom, t);
            }
        }
    }

    Color::from_rgb(0.0, 0.0, 0.0)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
    const SAMPLES_PER_PIXEL_SQRT: u32 = 4;
    const SAMPLES_PER_PIXEL_TOTAL: u32 = SAMPLES_PER_PIXEL_SQRT * SAMPLES_PER_PIXEL_SQRT;
    */
    const SAMPLES_PER_PIXEL_TOTAL: u32 = 100;

    let options = Options::parse();

    let image_width = options.width;
    let image_height = options.height;
    let aspect_ratio = image_width as f64 / image_height as f64;

    let camera = Camera::new(
        Vector::from_xyz(0.0, 0.0, 0.0),
        2.0 * aspect_ratio,
        2.0,
        1.0,
    );

    let ground: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Color::from_rgb(0.8, 0.8, 0.0),
    });
    let middle: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Color::from_rgb(0.7, 0.3, 0.3),
    });
    let left: Arc<dyn Material> = Arc::new(Metal {
        albedo: Color::from_rgb(0.8, 0.8, 0.8),
        fuzz: 0.3,
    });
    let right: Arc<dyn Material> = Arc::new(Metal {
        albedo: Color::from_rgb(0.8, 0.6, 0.2),
        fuzz: 1.0,
    });

    let mut world = World::new();
    world.add_geometry(Box::new(Sphere::new(
        Vector::from_xyz(0.0, -100.5, -1.0),
        100.0,
        Arc::clone(&ground),
    )));
    world.add_geometry(Box::new(Sphere::new(
        Vector::from_xyz(0.0, 0.0, -1.0),
        0.5,
        Arc::clone(&middle),
    )));
    world.add_geometry(Box::new(Sphere::new(
        Vector::from_xyz(-1.0, 0.0, -1.0),
        0.5,
        Arc::clone(&left),
    )));
    world.add_geometry(Box::new(Sphere::new(
        Vector::from_xyz(1.0, 0.0, -1.0),
        0.5,
        Arc::clone(&right),
    )));

    let pixels: Vec<u8> = (0..image_height)
        .into_par_iter()
        .rev()
        .flat_map(|y| {
            (0..image_width).into_par_iter().flat_map({
                let camera = &camera;
                let world = &world;
                move |x| {
                    let x = x as f64;
                    let y = y as f64;

                    let mut color_sum = Color::from_rgb(0.0, 0.0, 0.0);

                    /*
                    let offset_step = 1.0 / SAMPLES_PER_PIXEL_SQRT as f64;
                    for x_off in 0..SAMPLES_PER_PIXEL_SQRT {
                        for y_off in 0..SAMPLES_PER_PIXEL_SQRT {
                            let x_off = (x_off as f64 + 0.5) * offset_step;
                            let y_off = (y_off as f64 + 0.5) * offset_step;

                            let u = (x + x_off) / IMAGE_WIDTH as f64;
                            let v = (y + y_off) / IMAGE_HEIGHT as f64;

                            color_sum += ray_color(camera.get_ray(u, v), world, 50);
                        }
                    }
                    */

                    let mut rng = rand::thread_rng();
                    for _ in 0..SAMPLES_PER_PIXEL_TOTAL {
                        let u = (x + rng.gen::<f64>()) / image_width as f64;
                        let v = (y + rng.gen::<f64>()) / image_height as f64;
                        color_sum += ray_color_non_recursive(camera.get_ray(u, v), world, 50);
                    }

                    color_sum /= SAMPLES_PER_PIXEL_TOTAL as f64;

                    let inv_gamma = 1.0 / 2.2;
                    color_sum.r = color_sum.r.powf(inv_gamma);
                    color_sum.g = color_sum.g.powf(inv_gamma);
                    color_sum.b = color_sum.b.powf(inv_gamma);

                    color_sum.to_rgb_bytes()
                }
            })
        })
        .collect();

    let qoi = Qoi {
        width: image_width,
        height: image_height,
        colors: Colors::Srgb,
    };
    let encoded = qoi.encode_alloc(&pixels)?;

    std::fs::write("output.qoi", &encoded)?;

    Ok(())
}
