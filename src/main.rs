use std::path::PathBuf;

use argh::FromArgs;
use rapid_qoi::{Colors, Qoi};
use scene::Scene;

mod scene;

/// A blazingly slow toy CPU Raytracer
#[derive(FromArgs)]
struct Options {
    /// path to the scene file
    #[argh(positional)]
    scene: PathBuf,

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options: Options = argh::from_env();

    let scene = Scene::from_file(&options.scene)?;
    let camera = scene.construct_camera(options.width as f32 / options.height as f32);
    let world = scene.construct_world();

    let pixels = raybow::render(
        options.width,
        options.height,
        options.rays_per_pixel,
        &camera,
        &world,
        options.seed,
    );

    let qoi = Qoi {
        width: options.width,
        height: options.height,
        colors: Colors::Srgb,
    };
    let encoded = qoi.encode_alloc(&pixels)?;

    std::fs::write(options.output, &encoded)?;

    Ok(())
}
