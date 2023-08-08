use std::{
    cell::SyncUnsafeCell,
    io::Write,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    thread,
    time::{Duration, SystemTime},
};

use bumpalo::Bump;

use crate::{
    camera::Camera,
    color::Color,
    geometry::{bvh::Bvh, Object},
    image::Image,
    material::Reflection,
    philox::Philox4x32_10,
    ray::Ray,
};

#[repr(u32)]
pub enum RngKey {
    RayPixelOffset,
    CameraLensPosition,
    ScatterDirection,
    MetalFuzzDirection,
    RefractThreshold,
}

pub struct RayState {
    philox: Philox4x32_10,
    pixel_x: u32,
    pixel_y: u32,
    ray_number: u32,
    arena: Bump,
}

impl RayState {
    fn arena(&mut self) -> &mut Bump {
        &mut self.arena
    }

    pub fn gen_random_floats(&self, rng_key: RngKey) -> [f32; 4] {
        let ctr = [self.pixel_x, self.pixel_y, self.ray_number, rng_key as u32];
        self.philox.gen_f32s(ctr)
    }
}

pub struct RenderConfig<'a> {
    pub camera: &'a Camera,
    pub objects: Vec<Arc<dyn Object>>,
    pub background: Color,
    pub rays_per_pixel: u32,
    pub seed: u64,
    pub num_workers: usize,
}

pub fn render(config: RenderConfig<'_>, image: &mut Image) {
    let start_time = SystemTime::now();

    let image_width = image.width();
    let image_height = image.height();

    let bvh = Bvh::new(config.objects);

    let next_pixel = AtomicU32::new(0);
    let output: Vec<SyncUnsafeCell<Color>> =
        std::iter::repeat_with(|| SyncUnsafeCell::new(Color::BLACK))
            .take(image_width as usize * image_height as usize)
            .collect();

    thread::scope(|scope| {
        for _ in 0..config.num_workers {
            scope.spawn(|| unsafe {
                compute_pixels(
                    image_width,
                    image_height,
                    config.rays_per_pixel,
                    config.camera,
                    &bvh,
                    config.background,
                    config.seed,
                    &next_pixel,
                    &output,
                );
            });
        }
    });

    println!(
        "\x1B[G\x1B[KDone in {:.3?}",
        start_time.elapsed().unwrap_or(Duration::from_secs(0))
    );

    unsafe {
        let output: Vec<Color> = std::mem::transmute(output);
        image.pixels.copy_from_slice(&output);
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn compute_pixels(
    image_width: u32,
    image_height: u32,
    rays_per_pixel: u32,
    camera: &Camera,
    bvh: &Bvh<Vec<Arc<dyn Object>>>,
    background: Color,
    seed: u64,
    next_pixel: &AtomicU32,
    output: &[SyncUnsafeCell<Color>],
) {
    let mut state = RayState {
        philox: Philox4x32_10([(seed >> 32) as u32, seed as u32]),
        pixel_x: 0,
        pixel_y: 0,
        ray_number: 0,
        arena: Bump::new(),
    };

    loop {
        let pixel_number = next_pixel.fetch_add(1, Ordering::Relaxed);
        if pixel_number >= image_width * image_height {
            break;
        }

        let x = pixel_number % image_width;
        let y = pixel_number / image_width;

        if x == 0 {
            let mut stdout = std::io::stdout().lock();
            write!(
                stdout,
                "\x1B[G\x1B[K{}/{} ({:.0}%)",
                y,
                image_height,
                y as f32 / image_height as f32 * 100.0
            )
            .unwrap();
            stdout.flush().unwrap();
        }

        let mut color = Color::BLACK;

        state.pixel_x = x;
        state.pixel_y = y;
        for i in 0..rays_per_pixel {
            state.ray_number = i;

            let [x_off, y_off, ..] = state.gen_random_floats(RngKey::RayPixelOffset);

            let u = (x as f32 + x_off) / image_width as f32;
            let v = (y as f32 + y_off) / image_height as f32;
            let ray = camera.get_ray(1.0 - u, 1.0 - v, &state);

            color += ray_color(ray, bvh, 50, &mut state, background);

            state.arena().reset();
        }

        color /= rays_per_pixel as f32;

        unsafe {
            *output[pixel_number as usize].get() = color;
        }
    }
}

fn ray_color(
    mut ray: Ray,
    bvh: &Bvh<Vec<Arc<dyn Object>>>,
    max_bounces: u32,
    state: &mut RayState,
    background: Color,
) -> Color {
    let mut emitting = Color::BLACK;
    let mut attenuation = Color::WHITE;

    for _ in 0..max_bounces {
        state.arena().reset();
        match bvh.hit(ray, 0.0001..f32::INFINITY, state.arena()) {
            Some(hit) => {
                let material_hit = hit.material.hit(&hit, state);
                emitting += attenuation * material_hit.emission;
                match material_hit.reflection {
                    Some(Reflection {
                        ray: scatter_ray,
                        attenuation: attenuation_new,
                    }) => {
                        ray = scatter_ray;
                        attenuation *= attenuation_new;
                    }
                    None => break,
                }
            }
            None => {
                emitting += attenuation * background;
                break;
            }
        }
    }

    emitting
}
