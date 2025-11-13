use std::{
    io::Write,
    iter,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    thread,
    time::{Duration, SystemTime},
};

use bumpalo::Bump;

use crate::{
    camera::Camera,
    color::Color,
    geometry::{Object, bvh::Bvh},
    image::Image,
    material::Reflection,
    philox::Philox4x32_10,
    ray::Ray,
    sync_unsafe_cell::SyncUnsafeCell,
};

pub struct WorkerState {
    philox: Philox4x32_10,
    // The current pixel number
    pixel_number: u32,
    // The sample number of the current pixel
    sample_number: u32,
    // The ray number of the current sample
    ray_number: u32,
    rng_cnt: u32,
    arena: Bump,
}

impl WorkerState {
    fn new(seed: u64) -> Self {
        Self {
            philox: Philox4x32_10([(seed >> 32) as u32, seed as u32]),
            pixel_number: 0,
            sample_number: 0,
            ray_number: 0,
            rng_cnt: 0,
            arena: Bump::new(),
        }
    }

    fn arena(&mut self) -> &mut Bump {
        &mut self.arena
    }

    pub fn init_trace(&mut self, pixel_number: u32, sample_number: u32) {
        self.pixel_number = pixel_number;
        self.sample_number = sample_number;
        self.ray_number = 0;
        self.rng_cnt = 0;
    }

    pub fn gen_random_floats(&mut self) -> [f32; 4] {
        let ctr = [
            self.pixel_number,
            self.sample_number,
            self.ray_number,
            self.rng_cnt,
        ];
        self.rng_cnt += 1;
        self.philox.gen_f32s(ctr)
    }
}

pub struct RenderJob<'a> {
    pub camera: &'a Camera,
    pub objects: Vec<Arc<dyn Object>>,
    pub background: Color,
    pub num_samples: u32,
    pub seed: u64,
    pub num_workers: usize,
}

pub fn render(job: RenderJob<'_>, image: &mut Image) {
    let start_time = SystemTime::now();

    let image_width = image.width();
    let image_height = image.height();

    let bvh = Bvh::new(job.objects);

    let next_pixel = AtomicU32::new(0);
    let num_pixels = image_width as usize * image_height as usize;
    let output: Vec<_> = iter::repeat_with(|| SyncUnsafeCell::new(Color::BLACK))
        .take(num_pixels)
        .collect();

    thread::scope(|scope| {
        for _ in 0..job.num_workers {
            scope.spawn(|| unsafe {
                compute_pixels(
                    image_width,
                    image_height,
                    job.num_samples,
                    job.camera,
                    &bvh,
                    job.background,
                    job.seed,
                    &next_pixel,
                    &output,
                );
            });
        }
    });

    let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));
    println!("\x1B[G\x1B[KDone in {duration:.3?}",);

    unsafe {
        let output: Vec<Color> = std::mem::transmute(output);
        image.pixels.copy_from_slice(&output);
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn compute_pixels(
    image_width: u32,
    image_height: u32,
    num_samples: u32,
    camera: &Camera,
    bvh: &Bvh<Vec<Arc<dyn Object>>>,
    background: Color,
    seed: u64,
    next_pixel: &AtomicU32,
    output: &[SyncUnsafeCell<Color>],
) {
    let mut state = WorkerState::new(seed);

    loop {
        let pixel_number = next_pixel.fetch_add(1, Ordering::Relaxed);
        if pixel_number >= image_width * image_height {
            break;
        }

        let x = pixel_number % image_width;
        let y = pixel_number / image_width;

        if x == 0 {
            let mut stdout = std::io::stdout().lock();
            let progress = y as f32 / image_height as f32 * 100.0;
            write!(stdout, "\x1B[G\x1B[K{y}/{image_height} ({progress:.0}%)",).unwrap();
            stdout.flush().unwrap();
        }

        let mut color = Color::BLACK;

        for i in 0..num_samples {
            state.init_trace(pixel_number, i);

            let [x_off, y_off, ..] = state.gen_random_floats();

            let u = (x as f32 + x_off) / image_width as f32;
            let v = (y as f32 + y_off) / image_height as f32;
            let ray = camera.get_ray(1.0 - u, 1.0 - v, &mut state);

            color += ray_color(ray, bvh, 50, &mut state, background);

            state.arena().reset();
        }

        color /= num_samples as f32;

        unsafe {
            *output[pixel_number as usize].get() = color;
        }
    }
}

fn ray_color(
    mut ray: Ray,
    bvh: &Bvh<Vec<Arc<dyn Object>>>,
    max_bounces: u32,
    state: &mut WorkerState,
    background: Color,
) -> Color {
    let mut emitting = Color::BLACK;
    let mut attenuation = Color::WHITE;

    for _ in 0..max_bounces {
        state.arena().reset();
        state.ray_number += 1;
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
