use glam::{Mat4, Vec3};
use std::f32::consts::FRAC_PI_2;

pub struct Camera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new(target: Vec3, distance: f32, yaw: f32, pitch: f32) -> Self {
        Self {
            target,
            distance,
            yaw,
            pitch,
        }
    }

    pub fn eye(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + Vec3::new(x, y, z)
    }

    pub fn build_view_matrix(&self) -> Mat4 {
        let eye = self.eye();
        Mat4::look_at_lh(eye, self.target, Vec3::Y)
    }

    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch).clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance =
            (self.distance - delta).clamp(crate::constants::CAMERA_MIN_DISTANCE, crate::constants::CAMERA_MAX_DISTANCE);
    }

    pub fn pan(&mut self, delta_x: f32, delta_y: f32, speed: f32) {
        let forward = (self.target - self.eye()).normalize();
        let right = if forward.y.abs() > 0.99 {
            Vec3::X
        } else {
            Vec3::Y.cross(forward).normalize()
        };
        let up = forward.cross(right);
        self.target += right * delta_x * speed;
        self.target += up * delta_y * speed;
    }
}
