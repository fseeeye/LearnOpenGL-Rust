use nalgebra as na;

pub struct DirectionalLight {
    pub direction: na::Vector3<f32>,
    pub color: na::Vector3<f32>,
}

impl DirectionalLight {
    pub fn new(direction: na::Vector3<f32>, color: na::Vector3<f32>) -> Self {
        Self { direction, color }
    }
}

pub struct PointLight {
    pub position: na::Vector3<f32>,
    pub color: na::Vector3<f32>,

    pub attenuation_linear: f32,
    pub attenuation_quadratic: f32,
}

impl PointLight {
    pub fn new(
        position: na::Vector3<f32>,
        color: na::Vector3<f32>,
        attenuation_linear: f32,
        attenuation_quadratic: f32,
    ) -> Self {
        Self {
            position,
            color,
            attenuation_linear,
            attenuation_quadratic,
        }
    }
}

/// A kind of spot light which depends on the camera postion & direction
pub struct FlashLight {
    pub color: na::Vector3<f32>,

    pub cutoff: f32,
    pub outer_cutoff: f32,

    pub attenuation_linear: f32,
    pub attenuation_quadratic: f32,
}

impl FlashLight {
    pub fn new(
        color: na::Vector3<f32>,
        cutoff: f32,
        outer_cutoff: f32,
        attenuation_linear: f32,
        attenuation_quadratic: f32,
    ) -> Self {
        Self {
            color,
            cutoff,
            outer_cutoff,
            attenuation_linear,
            attenuation_quadratic,
        }
    }
}
