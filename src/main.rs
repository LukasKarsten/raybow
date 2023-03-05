use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use argh::FromArgs;
use rapid_qoi::{Colors, Qoi};
use raybow::{
    geometry::{Hittable, Sphere},
    material::{Dialectric, Lambertian, Material, Metal},
    vector::Vector,
    Camera, Color,
};
use scene::Scene;

mod scene;

/// A blazingly slow toy CPU Raytracer
#[derive(FromArgs)]
struct Options {
    /// path to the scene file
    #[argh(positional)]
    scene: String,

    /// width of the output image
    #[argh(positional)]
    width: u32,

    /// height of the output image
    #[argh(positional)]
    height: u32,

    /// number of rays that make up a single pixel
    #[argh(option, short = 'r', default = "500")]
    rays_per_pixel: u32,

    /// the seed
    #[argh(option, default = "0")]
    seed: u64,

    /// path to which the output should be written
    #[argh(option, short = 'o', default = "PathBuf::from(\"output.qoi\")")]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options: Options = argh::from_env();

    let (camera, objects) = match options.scene.as_str() {
        "builtin:spheres" => gen_scene_spheres(options.width as f32 / options.height as f32),
        _ => {
            let scene = Scene::from_file(Path::new(&options.scene))?;
            let camera = scene.construct_camera(options.width as f32 / options.height as f32);
            let objects = scene.construct_world();
            (camera, objects)
        }
    };

    let pixels = raybow::render(
        options.width,
        options.height,
        options.rays_per_pixel,
        &camera,
        objects,
        options.seed,
    );

    let qoi = Qoi {
        width: options.width,
        height: options.height,
        colors: Colors::Srgb,
    };
    let encoded = qoi.encode_alloc(&pixels)?;

    std::fs::write(options.output, encoded)?;

    Ok(())
}

fn gen_scene_spheres(aspect_ratio: f32) -> (Camera, Vec<Arc<dyn Hittable>>) {
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

    let mut objects = Vec::<Arc<dyn Hittable>>::new();

    let ground: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Color::from_rgb(0.5, 0.5, 0.5),
    });
    let floor_radius = 1000.0f32;
    objects.push(Arc::new(Sphere::new(
        Vector::from_xyz(0.0, -floor_radius, 0.0),
        floor_radius,
        Arc::clone(&ground),
    )));

    let num_spheres = 11;

    for a in -num_spheres..num_spheres {
        for b in -num_spheres..num_spheres {
            let x = a as f32;
            let z = b as f32;
            let y = (-x * x - z * z + floor_radius * floor_radius).sqrt() - floor_radius;
            let center = Vector::from_xyz(x, y + 0.2, z);

            let albedo = Color::from_rgb(0.3, 0.7, 0.9);
            let material = Arc::new(Lambertian { albedo });
            objects.push(Arc::new(Sphere::new(center, 0.2, material)));
        }
    }

    let material1: Arc<dyn Material> = Arc::new(Dialectric { index: 1.5 });
    objects.push(Arc::new(Sphere::new(
        Vector::from_xyz(0.0, 1.0, 0.0),
        1.0,
        Arc::clone(&material1),
    )));

    let material2: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Color::from_rgb(0.4, 0.2, 0.1),
    });
    objects.push(Arc::new(Sphere::new(
        Vector::from_xyz(-4.0, 1.0, 0.0),
        1.0,
        Arc::clone(&material2),
    )));

    let material3: Arc<dyn Material> = Arc::new(Metal {
        albedo: Color::from_rgb(0.7, 0.6, 0.5),
        fuzz: 0.0,
    });
    objects.push(Arc::new(Sphere::new(
        Vector::from_xyz(4.0, 1.0, 0.0),
        1.0,
        Arc::clone(&material3),
    )));

    (camera, objects)
}
