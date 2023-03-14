use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use argh::FromArgs;
use exr::image::{
    write::{channels::GetPixel, WritableImage},
    Image as ExrImage, SpecificChannels,
};
use rapid_qoi::{Colors, Qoi};
use raybow::{
    geometry::{Hittable, Sphere},
    material::{DiffuseLight, Lambertian, Material, Metal},
    vector::Vector,
    Camera, Color, Image,
};
use scene::Scene;

mod scene;

enum OutputFormat {
    Exr,
    Qoi,
    Png,
}

impl OutputFormat {
    fn default_file_extension(&self) -> &'static str {
        match self {
            Self::Exr => "exr",
            Self::Qoi => "qoi",
            Self::Png => "png",
        }
    }
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("exr") {
            Ok(Self::Exr)
        } else if s.eq_ignore_ascii_case("qoi") {
            Ok(Self::Qoi)
        } else if s.eq_ignore_ascii_case("png") {
            Ok(Self::Png)
        } else {
            Err(format!("unsupported output format: {s}"))
        }
    }
}

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
    #[argh(option, short = 'o')]
    output: Option<PathBuf>,

    /// data format in which to encode the output
    #[argh(option, short = 'f', default = "OutputFormat::Exr")]
    output_format: OutputFormat,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options: Options = argh::from_env();

    let (camera, objects, background) = match options.scene.as_str() {
        "builtin:spheres" => gen_scene_spheres(options.width as f32 / options.height as f32),
        _ => {
            let scene = Scene::from_file(Path::new(&options.scene))?;
            let camera = scene.construct_camera(options.width as f32 / options.height as f32);
            let objects = scene.construct_world();
            (camera, objects, scene.background)
        }
    };

    let mut image = Image::new(options.width, options.height);

    raybow::render(
        &mut image,
        options.rays_per_pixel,
        &camera,
        objects,
        background,
        options.seed,
    );

    let output_path = options.output.unwrap_or_else(|| {
        PathBuf::new()
            .with_file_name("output")
            .with_extension(options.output_format.default_file_extension())
    });

    match options.output_format {
        OutputFormat::Exr => write_exr(image, &output_path)?,
        OutputFormat::Qoi => write_qoi(image, &output_path)?,
        OutputFormat::Png => write_png(image, &output_path)?,
    }

    Ok(())
}

struct ImageGetPixelNewType<'a>(&'a Image);

impl<'a> GetPixel for ImageGetPixelNewType<'a> {
    type Pixel = (f32, f32, f32);

    fn get_pixel(&self, position: exr::prelude::Vec2<usize>) -> Self::Pixel {
        let x = position.x() as u32;
        let y = position.y() as u32;
        let Color { r, g, b } = self.0.pixel(x, y).unwrap();
        (r, g, b)
    }
}

fn write_exr(image: Image, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let pixels = SpecificChannels::rgb(ImageGetPixelNewType(&image));

    let exr_image =
        ExrImage::from_channels((image.width() as usize, image.height() as usize), pixels);

    exr_image.write().to_file(path)?;

    Ok(())
}

fn write_qoi(image: Image, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let qoi = Qoi {
        width: image.width(),
        height: image.height(),
        colors: Colors::Srgb,
    };

    let encoded = qoi.encode_alloc(&image.into_srgb_8bit())?;
    std::fs::write(path, encoded)?;

    Ok(())
}

fn write_png(image: Image, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use png::{AdaptiveFilterType, BitDepth, ColorType, Compression, SrgbRenderingIntent};

    let mut encoder = png::Encoder::new(
        BufWriter::new(File::create(path)?),
        image.width(),
        image.height(),
    );
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Sixteen);
    encoder.set_srgb(SrgbRenderingIntent::Perceptual);
    encoder.set_adaptive_filter(AdaptiveFilterType::Adaptive);
    encoder.set_compression(Compression::Best);

    encoder.add_text_chunk(String::from("software"), String::from("raybow"))?;

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&image.into_srgb_16bit())?;
    writer.finish()?;

    Ok(())
}

fn gen_scene_spheres(aspect_ratio: f32) -> (Camera, Vec<Arc<dyn Hittable>>, Color) {
    let lookfrom = Vector::from_xyz(13.0, 2.0, 3.0);
    let lookat = Vector::from_xyz(0.0, 0.0, 0.0);
    let vup = Vector::from_xyz(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;

    let camera = Camera::new(
        lookfrom,
        lookat,
        vup,
        50.0,
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
            let material = Arc::new(Metal { albedo, fuzz: 0.1 });
            objects.push(Arc::new(Sphere::new(center, 0.2, material)));
        }
    }

    let light: Arc<dyn Material> = Arc::new(DiffuseLight {
        emit: Color::WHITE * 4.0,
    });
    objects.push(Arc::new(Sphere::new(
        Vector::from_xyz(0.0, 3.0, 0.0),
        0.5,
        Arc::clone(&light),
    )));

    (camera, objects, Color::BLACK)
}
