use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use geometry::BvhNode;
use indicatif::ParallelProgressIterator;
use material::Dialectric;
use rand::Rng;
use rapid_qoi::{Colors, Qoi};
use ray::Ray;
use rayon::prelude::*;

use crate::{
    camera::Camera,
    color::Color,
    geometry::{Hittable, Sphere},
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
    #[clap(short, long, default_value_t = 500)]
    rays_per_pixel: u32,
    #[clap(short, long, default_value = "output.qoi")]
    output: PathBuf,
    #[clap(long, default_value_t = 22)]
    spheres_per_axis: u32,
}

fn ray_color_non_recursive(ray: Ray, geometry: &impl Hittable, max_bounces: u32) -> Color {
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
    let options = Options::parse();

    let image_width = options.width;
    let image_height = options.height;
    let aspect_ratio = image_width as f64 / image_height as f64;
    let rays_per_pixel = options.rays_per_pixel;

    let lookfrom = Vector::from_xyz(13.0, 2.0, 3.0);
    let lookat = Vector::from_xyz(0.0, 0.0, 0.0);
    let vup = Vector::from_xyz(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;

    let camera = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.0,
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    let mut world = BvhNode::new();

    let ground: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Color::from_rgb(0.5, 0.5, 0.5),
    });
    world.push(Box::new(Sphere::new(
        Vector::from_xyz(0.0, -1000.0, 0.0),
        1000.0,
        Arc::clone(&ground),
    )));

    let mut rng = rand::thread_rng();

    let num_spheres = (options.spheres_per_axis / 2) as i32;

    for a in -num_spheres..num_spheres {
        for b in -num_spheres..num_spheres {
            let x = a as f64 + 0.9 * rng.gen::<f64>();
            let z = b as f64 + 0.9 * rng.gen::<f64>();
            let center = Vector::from_xyz(x, 0.2, z);

            if (center - Vector::from_xyz(4.0, 0.2, 0.0)).length() > 0.9 {
                let choose_mat: f64 = rng.gen();

                if choose_mat < 0.8 {
                    let albedo = Color::random() * Color::random();
                    let material = Arc::new(Lambertian { albedo });
                    world.push(Box::new(Sphere::new(center, 0.2, material)));
                } else if choose_mat < 0.95 {
                    let albedo = Color::random();
                    let fuzz = rng.gen_range(0.0..0.5);
                    let material = Arc::new(Metal { albedo, fuzz });
                    world.push(Box::new(Sphere::new(center, 0.2, material)))
                } else {
                    let material = Arc::new(Dialectric { index: 1.5 });
                    world.push(Box::new(Sphere::new(center, 0.2, material)))
                }
            }
        }
    }

    let material1: Arc<dyn Material> = Arc::new(Dialectric { index: 1.5 });
    world.push(Box::new(Sphere::new(
        Vector::from_xyz(0.0, 1.0, 0.0),
        1.0,
        Arc::clone(&material1),
    )));

    let material2: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Color::from_rgb(0.4, 0.2, 0.1),
    });
    world.push(Box::new(Sphere::new(
        Vector::from_xyz(-4.0, 1.0, 0.0),
        1.0,
        Arc::clone(&material2),
    )));

    let material3: Arc<dyn Material> = Arc::new(Metal {
        albedo: Color::from_rgb(0.7, 0.6, 0.5),
        fuzz: 0.1,
    });
    world.push(Box::new(Sphere::new(
        Vector::from_xyz(4.0, 1.0, 0.0),
        1.0,
        Arc::clone(&material3),
    )));

    let pixels: Vec<u8> = (0..image_height)
        .into_par_iter()
        .rev()
        .progress()
        .flat_map(|y| {
            (0..image_width).into_par_iter().flat_map({
                let camera = &camera;
                let world = &world;
                move |x| {
                    let x = x as f64;
                    let y = y as f64;

                    let mut color_sum = Color::from_rgb(0.0, 0.0, 0.0);

                    let mut rng = rand::thread_rng();
                    for _ in 0..rays_per_pixel {
                        let u = (x + rng.gen::<f64>()) / image_width as f64;
                        let v = (y + rng.gen::<f64>()) / image_height as f64;
                        color_sum += ray_color_non_recursive(camera.get_ray(u, v), world, 50);
                    }

                    color_sum /= rays_per_pixel as f64;

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

    std::fs::write(options.output, &encoded)?;

    Ok(())
}
