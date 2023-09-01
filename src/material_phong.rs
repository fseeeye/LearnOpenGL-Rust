use crate::Texture;

#[derive(Debug)]
pub struct MaterialPhong {
    pub diffuse_map: Texture,
    pub specular_map: Texture,
    pub shininess: f32,
    pub emission_map: Option<Texture>,
}

impl MaterialPhong {
    pub fn new(
        diffuse_map: Texture,
        specular_map: Texture,
        shininess: f32,
        emission_map: Option<Texture>,
    ) -> MaterialPhong {
        MaterialPhong {
            diffuse_map,
            specular_map,
            shininess,
            emission_map,
        }
    }
}
