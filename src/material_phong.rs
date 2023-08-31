use nalgebra as na;

#[derive(Debug)]
pub struct MaterialPhong {
    pub diffuse_coefficient: na::Vector3<f32>,
    pub specular_coefficient: na::Vector3<f32>,
    pub ambient_coefficient: na::Vector3<f32>,
    pub shininess: f32,
}

impl MaterialPhong {
    pub fn new(
        diffuse_coefficient: na::Vector3<f32>,
        specular_coefficient: na::Vector3<f32>,
        ambient_coefficient: na::Vector3<f32>,
        shininess: f32,
    ) -> MaterialPhong {
        MaterialPhong {
            diffuse_coefficient,
            specular_coefficient,
            ambient_coefficient,
            shininess,
        }
    }
}
