use nalgebra as na;

use crate::Camera;

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

pub struct SpotLight<'a> {
    pub camera: &'a Camera,

    pub color: na::Vector3<f32>,

    pub cutoff: f32,
    pub outer_cutoff: f32,

    pub attenuation_linear: f32,
    pub attenuation_quadratic: f32,
}

impl<'a> SpotLight<'a> {
    pub fn new(
        camera: &'a Camera,
        color: na::Vector3<f32>,
        cutoff: f32,
        outer_cutoff: f32,
        attenuation_linear: f32,
        attenuation_quadratic: f32,
    ) -> Self {
        Self {
            camera,
            color,
            cutoff,
            outer_cutoff,
            attenuation_linear,
            attenuation_quadratic,
        }
    }
}
