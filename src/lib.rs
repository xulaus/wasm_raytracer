mod utils;
use std::ops;

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

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

#[wasm_bindgen]
pub struct RenderState {
    width: u32,
    height: u32,
    img_data: Vec<u8>,
    last_frame: u32,
    random: XoShiRo256,
    camera: Vec3,
    camera_target: Vec3,
}

#[wasm_bindgen]
pub fn setup(width: u32, height: u32) -> RenderState {
    utils::set_panic_hook();
    log("Setup complete");
    RenderState {
        width,
        height,
        img_data: vec![0xAA; (4 * width * height) as usize],
        last_frame: 0,
        camera: Vec3 {
            x: -10.0,
            y: -10.0,
            z: -30.0
        },
        camera_target: Vec3 {
            x: -1.0,
            y: -1.0,
            z: 3.0,
        },
        random: XoShiRo256 {
            state: [31415, 27182, 141142, 17320],
        },
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
            z: self.x * other.y - self.y * other.x
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
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

struct Sphere {
    c: Vec3,
    r: f32
}

fn intersect_with_sphere(ray_start: &Vec3, ray_dir: &Vec3, s: &Sphere) -> Option<Vec3> {
    let adj_start = *ray_start - s.c;
    let v_d = Vec3::dot(&adj_start, &ray_dir);
    let view_sqr = Vec3::dot(&adj_start, &adj_start);
    let chord = (v_d * v_d) - (view_sqr - s.r*s.r);
    if chord >= 0.0  && v_d <= 0.0 {
       return Some(*ray_start - (v_d + chord.sqrt()) * *ray_dir)
    } else {
        None
    }
}

#[wasm_bindgen]
impl RenderState {
    pub fn img(&self) -> *const u8 {
        self.img_data.as_ptr()
    }

    pub fn set_camera_to(&mut self, x: f32, y: f32, z: f32) {
        self.camera_target = Vec3 {x, y, z};
    }
    pub fn tick(&mut self) {
        let diff = self.camera - self.camera_target;
        self.camera = if diff.abs() < 0.01 {
            self.camera_target
        } else {
            self.camera + diff * 0.1
        };
        let target = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let up = Vec3 { x: 0.0, y: 1.0, z: 0.0};

        let t = (target - self.camera).norm();
        let b = Vec3::cross(&up, &t).norm();
        let v = Vec3::cross(&t, &b);
        // fov = (pi / 2)
        //
        let d = 0.01; // distance from focal point

        //shift vectors
        let width: f32 = self.width as f32;
        let height: f32 = self.height as f32;
        let gx = d;
        let gy = (gx * height) / width;
        let shift_x = -((2.0 * gx) / (width - 1.0)) * b;
        let shift_y = -((2.0 * gy) / (height - 1.0)) * v;

        let veiw_0 = d * t - gx * b - gy * v;

        let light = Sphere {
            c: Vec3 { x: 2.0, y: 2.0, z: 0.5 },
            r: 0.5,
        };

        let orb = Sphere {
            c: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            r: 1.0,
        };
        for y in 0..self.height {
            for x in 0..self.width {
                let cur_veiw = veiw_0 + shift_x * x + shift_y * y;
                let index = ((x + y * self.width) * 4) as usize;
                let ray_dir = cur_veiw.norm();
                let col = if let Some(_) = intersect_with_sphere(&self.camera, &ray_dir, &light) {
                    0xFF
                } else if let Some(y) = intersect_with_sphere(&self.camera, &ray_dir, &orb) {
                    let normal = (y - orb.c).norm();
                    let reflect = (ray_dir - Vec3::dot(&ray_dir, &normal) * normal).norm();
                    if let Some(_) = intersect_with_sphere(&y, &reflect, &light) {
                        0xCC
                    } else {
                        0x00
                    }
                } else { 0xAA };
                self.img_data[index] = col;
                self.img_data[index + 1] = col;
                self.img_data[index + 2] = col;
                self.img_data[index + 3] = 0xFF;
            }
        }
        self.last_frame += 1;
    }
}
