use std::ops;

const EPSILON: f32 = 0.001;

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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn norm(&self) -> Vec3 {
        let abs = self.abs();
        Vec3 {
            x: self.x / abs,
            y: self.y / abs,
            z: self.z / abs,
        }
    }

    pub fn abs(&self) -> f32 {
        Vec3::dot(self, self).sqrt()
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
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
pub struct Sphere {
    pub c: Vec3,
    pub r: f32,
}
impl Sphere {
    pub fn intersect_with(&self, ray: &Line) -> Option<f32> {
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
pub struct Plane {
    pub p: Vec3, // A point on the plane
    pub n: Vec3, // Normal to the plane
}

impl Plane {
    pub fn intersect_with(&self, ray: &Line) -> Option<f32> {
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
pub struct Line {
    pub start: Vec3,
    pub dir: Vec3,
}

impl Line {
    pub fn point_at(&self, d: f32) -> Vec3 {
        self.start + d * self.dir
    }
}
