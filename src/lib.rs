#![feature(sync_unsafe_cell, core_intrinsics)]

use std::{
    cell::SyncUnsafeCell,
    io::Write,
    mem::ManuallyDrop,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    thread,
    time::{Duration, SystemTime},
};

use bumpalo::Bump;
use geometry::Hittable;

use crate::{geometry::bvh::LinearTree, philox::Philox4x32_10};

use self::ray::Ray;

pub use self::{camera::Camera, color::Color};

mod camera;
mod color;
mod philox;
mod ray;

pub mod geometry;
pub mod material;
pub mod vector;

#[repr(u32)]
enum RngKey {
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

    fn gen_random_floats(&self, rng_key: RngKey) -> [f32; 4] {
        let ctr = [self.pixel_x, self.pixel_y, self.ray_number, rng_key as u32];
        self.philox.gen_f32s(ctr)
    }
}

fn ray_color(ray: Ray, bvh: &LinearTree, max_bounces: u32, state: &mut RayState) -> Color {
    let mut color = Color::from_rgb(1.0, 1.0, 1.0);

    let mut curr_ray = ray;

    for _ in 0..max_bounces {
        match bvh.hit(curr_ray, state.arena()) {
            Some(hit) => match hit.material.scatter(&hit, state) {
                Some((ray, attenuation)) => {
                    color *= attenuation;
                    curr_ray = ray;
                }
                None => return Color::from_rgb(0.0, 0.0, 0.0),
            },
            None => {
                let unit_vel = curr_ray.velocity.normalize_unchecked();

                let t = 0.5 * (unit_vel.y() + 1.0f32);
                let top = Color::from_rgb(1.0, 1.0, 1.0);
                let bottom = Color::from_rgb(0.5, 0.7, 1.0);
                return color * top.lerp(bottom, t);
            }
        }
    }

    Color::from_rgb(0.0, 0.0, 0.0)
}

pub fn render(
    image_width: u32,
    image_height: u32,
    rays_per_pixel: u32,
    camera: &Camera,
    objects: Vec<Arc<dyn Hittable>>,
    seed: u64,
) -> Vec<u8> {
    let start_time = SystemTime::now();

    let bvh = LinearTree::new(objects);

    let cpus = num_cpus::get();

    let next_pixel = AtomicU32::new(0);
    //let finished_pixels = AtomicU32::new(0);
    let output: ManuallyDrop<Vec<SyncUnsafeCell<u8>>> = ManuallyDrop::new(
        std::iter::repeat_with(|| SyncUnsafeCell::new(0))
            .take(image_width as usize * image_height as usize * 3)
            .collect(),
    );

    thread::scope(|scope| {
        for _ in 0..cpus {
            let output = &output;
            let next_pixel = &next_pixel;
            let bvh = &bvh;

            let mut state = RayState {
                philox: Philox4x32_10([(seed >> 32) as u32, seed as u32]),
                pixel_x: 0,
                pixel_y: 0,
                ray_number: 0,
                arena: Bump::new(),
            };

            scope.spawn(move || loop {
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

                let mut color_sum = Color::from_rgb(0.0, 0.0, 0.0);

                state.pixel_x = x;
                state.pixel_y = y;
                for i in 0..rays_per_pixel {
                    state.ray_number = i;

                    let [x_off, y_off, ..] = state.gen_random_floats(RngKey::RayPixelOffset);

                    let u = (x as f32 + x_off) / image_width as f32;
                    let v = (y as f32 + y_off) / image_height as f32;
                    let ray = camera.get_ray(u, 1.0 - v, &state);

                    color_sum += ray_color(ray, bvh, 50, &mut state);

                    state.arena().reset();
                }

                color_sum /= rays_per_pixel as f32;

                let inv_gamma = 1.0 / 2.2;
                color_sum.r = color_sum.r.powf(inv_gamma);
                color_sum.g = color_sum.g.powf(inv_gamma);
                color_sum.b = color_sum.b.powf(inv_gamma);

                let color = color_sum.to_rgb_bytes();

                unsafe {
                    let off = pixel_number as usize * 3;
                    output[off + 0].get().write(color[0]);
                    output[off + 1].get().write(color[1]);
                    output[off + 2].get().write(color[2]);
                }
            });
        }
    });

    println!(
        "\x1B[G\x1B[KDone in {:.3?}",
        start_time.elapsed().unwrap_or(Duration::from_secs(0))
    );

    unsafe {
        let ptr = output.as_ptr();
        let len = output.len();
        let cap = output.capacity();
        drop(output);
        Vec::from_raw_parts(ptr as _, len, cap)
    }
}
