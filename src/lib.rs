mod geometry;
mod random_seq;
mod utils;

extern crate wasm_bindgen;

use geometry::{Line, Plane, Sphere, Vec3};
use random_seq::RandomSeq;
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

const EPSILON: f32 = 0.001;

#[derive(Debug)]
struct RayCastJob {
    ray: Line,
    pixel: usize,
    alpha: u8,
}

#[wasm_bindgen]
pub struct RenderState {
    width: u32,
    height: u32,
    img_data: Vec<u8>,
    random: RandomSeq,
    camera: Vec3,
    camera_target: Vec3,
    active_rays: VecDeque<RayCastJob>,
    rays_per_pixel: u8,
}

#[wasm_bindgen]
impl RenderState {
    pub fn img(&self) -> *const u8 {
        self.img_data.as_ptr()
    }

    pub fn active_rays(&self) -> usize {
        self.active_rays.len()
    }

    pub fn set_camera_to(&mut self, x: f32, y: f32, z: f32) {
        self.camera_target = Vec3 { x, y, z };
    }

    fn clear_screen(&mut self) {
        self.active_rays.clear();
        for elem in self.img_data.iter_mut() {
            *elem = 0x00;
        }
        self.rays_per_pixel = 0;
    }

    fn cast_from_camera(&mut self, alpha: u8) {
        let target = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let up = Vec3 {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        };

        let t = (target - self.camera).norm();
        let b = Vec3::cross(&up, &t).norm();
        let v = Vec3::cross(&t, &b);
        // fov = (pi / 2)
        //
        let d = EPSILON; // distance from focal point

        //shift vectors
        let width: f32 = self.width as f32;
        let height: f32 = self.height as f32;
        let gx = d;
        let gy = (gx * height) / width;
        let shift_x = ((2.0 * gx) / (width - 1.0)) * b;
        let shift_y = ((2.0 * gy) / (height - 1.0)) * v;

        let veiw_0 = d * t - gx * b - gy * v;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.random.next() & 0xFFFF;
                let idy = self.random.next() & 0xFFFF;
                let dx = (idx as f32) / (0xFFFF as f32) - 0.5;
                let dy = (idy as f32) / (0xFFFF as f32) - 0.5;
                let cur_veiw = veiw_0 + shift_x * (x as f32 + dx) + shift_y * (y as f32 + dy);
                let pixel = ((x + y * self.width) * 4) as usize;
                let ray = Line {
                    start: self.camera,
                    dir: cur_veiw.norm(),
                };
                self.active_rays.push_back(RayCastJob {
                    ray,
                    pixel,
                    alpha: alpha,
                });
            }
        }
    }

    pub fn tick(&mut self) {
        let diff = self.camera - self.camera_target;
        if diff.abs() != 0.0 {
            self.camera = if diff.abs() < 0.1 {
                self.camera_target
            } else {
                self.camera - diff * 0.1
            };
            self.clear_screen();
            self.cast_from_camera(0xFF);
        }

        let floor = Plane {
            p: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            n: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        };
        let light = Sphere {
            c: Vec3 {
                x: 2.0,
                y: 2.0,
                z: 0.0,
            },
            r: 0.25,
        };

        let orb = Sphere {
            c: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            r: 1.0,
        };

        fn reflect_in_sphere(s: &Sphere, ray: &Line, interestion_at: f32) -> Line {
            let y = ray.point_at(interestion_at);
            let normal = (y - s.c).norm();
            let reflect = (ray.dir - 2.0 * Vec3::dot(&ray.dir, &normal) * normal).norm();
            Line {
                start: y,
                dir: reflect,
            }
        }

        fn intersect_before(a: Option<f32>, b: Option<f32>) -> bool {
            if let Some(a_) = a {
                if let Some(b_) = b {
                    a_ < b_
                } else {
                    true
                }
            } else {
                false
            }
        }

        let rays_per_frame: u32 = self.height * self.width * 2;
        for _ in 0..rays_per_frame {
            if let Some(job) = self.active_rays.pop_front() {
                let ray = &job.ray;
                let pixel = job.pixel;
                let sphere_col = orb.intersect_with(&ray);
                let light_col = light.intersect_with(&ray);
                let floor_col = floor.intersect_with(&ray);
                let (col, alpha) = if intersect_before(sphere_col, light_col) {
                    let ray_alpha = job.alpha / 2;
                    if ray_alpha != 0 {
                        self.active_rays.push_back(RayCastJob {
                            ray: reflect_in_sphere(&orb, &ray, sphere_col.unwrap()),
                            pixel,
                            alpha: ray_alpha,
                        });
                    }
                    let absorbsion_alpha = job.alpha - ray_alpha;

                    (0xAA, absorbsion_alpha)
                // } else if let Some(_) = light1.intersect_with(&ray) {
                //     (0xFF, job.alpha)
                } else if intersect_before(light_col, sphere_col) {
                    (0xFF, job.alpha)
                } else if let Some(d) = floor_col {
                    let point = ray.point_at(d);
                    let dx = (point.x.fract() + 1.0).fract();
                    let dy = (point.z.fract() + 1.0).fract();
                    let black = dx < 0.5 && dy < 0.5 || dy > 0.5 && dx > 0.5;
                    let col = if black { 0xDD } else { 0x22 };
                    let ray_alpha = job.alpha / 8;
                    if ray_alpha != 0 {
                        let reflect =
                            (ray.dir - 2.0 * Vec3::dot(&floor.n, &ray.dir) * floor.n).norm();
                        self.active_rays.push_back(RayCastJob {
                            ray: Line {
                                start: point,
                                dir: reflect,
                            },
                            pixel,
                            alpha: ray_alpha,
                        });
                    }
                    let absorbsion_alpha = job.alpha - ray_alpha;

                    (col, absorbsion_alpha)
                } else {
                    (0x00, job.alpha)
                };

                fn col_lerp(val: u8, alpha: u8, col: u8) -> u8 {
                    let promoted_val: u32 = val as u32;
                    let promoted_alpha: u32 = alpha as u32;
                    let promoted_col: u32 = col as u32;
                    let new_col = (promoted_val * (255 - promoted_alpha)
                        + (promoted_col * promoted_alpha))
                        / 255;
                    new_col as u8
                }

                self.img_data[pixel + 0] = col_lerp(self.img_data[pixel + 0], alpha, col);
                self.img_data[pixel + 1] = col_lerp(self.img_data[pixel + 1], alpha, col);
                self.img_data[pixel + 2] = col_lerp(self.img_data[pixel + 2], alpha, col);
                self.img_data[pixel + 3] = 0xFF;
            }
        }
        if self.active_rays.len() == 0 && self.rays_per_pixel < 32 {
            self.rays_per_pixel += 1;
            self.cast_from_camera(0xFF / self.rays_per_pixel)
        }
    }
}

#[wasm_bindgen]
pub fn setup(width: u32, height: u32) -> RenderState {
    utils::set_panic_hook();
    let mut ret = RenderState {
        width,
        height,
        img_data: vec![0x00; (4 * width * height) as usize],
        camera: Vec3 {
            x: -10.0,
            y: 5.0,
            z: -30.0,
        },
        camera_target: Vec3 {
            x: 3.0,
            y: 3.0,
            z: 3.0,
        },
        random: RandomSeq::new(),
        active_rays: VecDeque::new(),
        rays_per_pixel: 1,
    };
    ret.clear_screen();
    ret.cast_from_camera(0xFF);
    log("Setup complete");
    ret
}
