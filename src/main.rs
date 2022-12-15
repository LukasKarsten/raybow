use std::{path::PathBuf, sync::Arc};

use argh::FromArgs;
use rapid_qoi::{Colors, Qoi};

use raybow::{
    geometry::{Sphere, World},
    material::{Dialectric, Lambertian, Material, Metal},
    vector::Vector,
    Camera, Color,
};

/// A blazingly slow toy CPU Raytracer
#[derive(FromArgs)]
struct Options {
    /// width of the output image
    #[argh(positional)]
    width: u32,

    /// height of the output image
    #[argh(positional)]
    height: u32,

    /// number of rays that make up a single pixel
    #[argh(option, short = 'r', default = "500")]
    rays_per_pixel: u32,

    /// path to which the output should be written
    #[argh(option, short = 'o', default = "PathBuf::from(\"output.qoi\")")]
    output: PathBuf,

    /// the seed
    #[argh(option, default = "0")]
    seed: u64,

    #[argh(option, hidden_help, default = "22")]
    spheres_per_axis: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options: Options = argh::from_env();

    let image_width = options.width;
    let image_height = options.height;
    let aspect_ratio = image_width as f32 / image_height as f32;
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

    let mut world = World::new();

    let ground: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Color::from_rgb(0.5, 0.5, 0.5),
    });
    let floor_radius = 1000.0f32;
    world.push(Box::new(Sphere::new(
        Vector::from_xyz(0.0, -floor_radius, 0.0),
        floor_radius,
        Arc::clone(&ground),
    )));

    let num_spheres = (options.spheres_per_axis / 2) as i32;

    for a in -num_spheres..num_spheres {
        for b in -num_spheres..num_spheres {
            let x = a as f32;
            let z = b as f32;
            let y = (-x * x - z * z + floor_radius * floor_radius).sqrt() - floor_radius;
            let center = Vector::from_xyz(x, y + 0.2, z);

            if (center - Vector::from_xyz(4.0, 0.2, 0.0)).length() > 0.9 {
                let albedo = Color::from_rgb(0.3, 0.7, 0.9);
                let material = Arc::new(Lambertian { albedo });
                world.push(Box::new(Sphere::new(center, 0.2, material)));
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
        fuzz: 0.0,
    });
    world.push(Box::new(Sphere::new(
        Vector::from_xyz(4.0, 1.0, 0.0),
        1.0,
        Arc::clone(&material3),
    )));

    let pixels = raybow::render(
        image_width,
        image_height,
        rays_per_pixel,
        &camera,
        &world,
        options.seed,
    );

    let qoi = Qoi {
        width: image_width,
        height: image_height,
        colors: Colors::Srgb,
    };
    let encoded = qoi.encode_alloc(&pixels)?;

    std::fs::write(options.output, &encoded)?;

    Ok(())
}
