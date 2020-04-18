mod utils;
use std::{collections::VecDeque, ops};
macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = min!($($z),*);
        if $x < y {
            $x
        } else {
            y
        }
    }}
}

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

const EPSILON: f32 = 0.001;

#[derive(Clone)]
pub struct XoShiRo256 {
    state: [u64; 4],
}

impl XoShiRo256 {
    pub fn next(&mut self) -> u64 {
        fn rol64(x: u64, k: u64) -> u64 {
            (x << k) | (x >> (64 - k))
        }
        let result = rol64(self.state[1] * 5, 7) * 9;
        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = rol64(self.state[3], 45);

        return result;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn norm(&self) -> Vec3 {
        let abs = self.abs();
        Vec3 {
            x: self.x / abs,
            y: self.y / abs,
            z: self.z / abs,
        }
    }

    fn abs(&self) -> f32 {
        Vec3::dot(self, self).sqrt()
    }

    fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl ops::Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        rhs * self
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, scale: f32) -> Vec3 {
        Vec3 {
            x: scale * self.x,
            y: scale * self.y,
            z: scale * self.z,
        }
    }
}

impl ops::Mul<u32> for Vec3 {
    type Output = Vec3;
    fn mul(self, i_scale: u32) -> Vec3 {
        let scale = i_scale as f32;
        Vec3 {
            x: scale * self.x,
            y: scale * self.y,
            z: scale * self.z,
        }
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

#[derive(Debug)]
struct Sphere {
    c: Vec3,
    r: f32,
}
impl Sphere {
    fn intersect_with(&self, ray: &Line) -> Option<f32> {
        let adj_start = ray.start - self.c;
        let v_d = Vec3::dot(&adj_start, &ray.dir);
        let view_sqr = Vec3::dot(&adj_start, &adj_start);
        let chord = (v_d * v_d) - (view_sqr - self.r * self.r);

        if chord >= 0.0 {
            let d1 = -v_d + chord.sqrt();
            let d2 = -v_d - chord.sqrt();
            if d1 <= EPSILON && d2 <= EPSILON {
                None
            } else if d1 <= EPSILON {
                Some(d2)
            } else if d2 <= EPSILON {
                Some(d1)
            } else {
                Some(min!(d1, d2))
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct Plane {
    p: Vec3, // A point on the plane
    n: Vec3, // Normal to the plane
}

impl Plane {
    fn intersect_with(&self, ray: &Line) -> Option<f32> {
        let cos = Vec3::dot(&ray.dir, &self.n);
        if cos != 0.0 {
            let ret = Vec3::dot(&(self.p - ray.start), &self.n) / cos;
            if ret > EPSILON {
                return Some(ret);
            }
        }

        None
    }
}

#[derive(Debug)]
struct Line {
    start: Vec3,
    dir: Vec3,
}

impl Line {
    fn point_at(&self, d: f32) -> Vec3 {
        self.start + d * self.dir
    }
}

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
    random: XoShiRo256,
    camera: Vec3,
    camera_target: Vec3,
    active_rays: VecDeque<RayCastJob>,
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

    fn create_init_rays(&mut self) {
        self.active_rays.clear();
        for elem in self.img_data.iter_mut() {
            *elem = 0x00;
        }

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
                let cur_veiw = veiw_0 + shift_x * x + shift_y * y;
                let pixel = ((x + y * self.width) * 4) as usize;
                let ray = Line {
                    start: self.camera,
                    dir: cur_veiw.norm(),
                };
                self.active_rays.push_back(RayCastJob {
                    ray,
                    pixel,
                    alpha: 0xFF,
                });
            }
        }
    }

    pub fn tick(&mut self) {
        let diff = self.camera - self.camera_target;
        if diff.abs() != 0.0 {
            self.camera = if diff.abs() < 0.01 {
                self.camera_target
            } else {
                self.camera - diff * 0.1
            };
            self.create_init_rays();
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

        for _ in 0..2500000 {
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
                    let mut r = point.x.abs() as i32 + point.z.abs() as i32;
                    if point.x <= 0.0 {
                        r+=1;
                    }
                    if point.z <= 0.0 {
                        r+=1;
                    }
                    let col = if r % 2 == 0 {
                        0xCC
                    } else {
                        0x22
                    };
                    let ray_alpha = job.alpha / 8;
                    if ray_alpha != 0 {
                        let reflect = (ray.dir - 2.0 * Vec3::dot(&floor.n, &ray.dir) * floor.n).norm();
                        self.active_rays.push_back(RayCastJob {
                            ray: Line { start: point, dir: reflect },
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
                self.img_data[pixel + 3] += 0xFF;
            }
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
            x: 3.0 ,
            y: 3.0,
            z: 3.0,
        },
        random: XoShiRo256 {
            state: [31415, 27182, 141142, 17320],
        },
        active_rays: VecDeque::new(),
    };
    ret.create_init_rays();
    log("Setup complete");
    ret
}
