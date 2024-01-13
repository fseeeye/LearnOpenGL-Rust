use std::path::PathBuf;

use anyhow::{bail, Ok};
use nalgebra as na;
use tracing::{debug, trace, warn};

use crate::{Mesh, ShaderProgram, Texture, TextureType, Vertex};

pub struct Model {
    pub meshes: Vec<Mesh>,
    textures_loaded: Vec<Texture>,
    model_path: PathBuf,
}

impl Model {
    pub fn new(model_path: PathBuf) -> anyhow::Result<Model> {
        let mut model = Self {
            meshes: Vec::new(),
            textures_loaded: Vec::new(),
            model_path,
        };

        model.load_model()?;

        Ok(model)
    }

    pub fn draw(&self, shader: &ShaderProgram, material_uniform_name: &str) -> anyhow::Result<()> {
        for mesh in self.meshes.iter() {
            mesh.draw(shader, material_uniform_name)?;
        }

        Ok(())
    }

    fn load_model(&mut self) -> anyhow::Result<()> {
        debug!("Loading model from {:?}", &self.model_path);

        // Load .obj file
        let (models, materials) = tobj::load_obj(
            &self.model_path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ignore_points: true,
                ignore_lines: true,
            },
        )?;
        let materials = materials?;

        trace!("Number of models          = {}", models.len());
        trace!("Number of materials       = {}", materials.len());

        // Load Meshes
        for (i, model) in models.iter().enumerate() {
            trace!("model[{}].name             = \'{}\'", i, model.name);
            trace!(
                "model[{}].mesh.material_id = {:?}",
                i,
                model.mesh.material_id
            );
            trace!(
                "model[{}].face_count       = {}",
                i,
                model.mesh.face_arities.len()
            );

            let mesh = self.load_mesh(&model.mesh, &materials, &model.name)?;
            self.meshes.push(mesh);
        }

        Ok(())
    }

    fn load_mesh(
        &mut self,
        mesh: &tobj::Mesh,
        materials: &[tobj::Material],
        model_name: &str,
    ) -> anyhow::Result<Mesh> {
        // Handle indices of mesh
        let indices: Vec<u32> = mesh.indices.clone();

        // Handle vertices of mesh
        assert!(mesh.positions.len() % 3 == 0);
        let vertices_num = mesh.positions.len() / 3;
        let mut vertices: Vec<Vertex> = Vec::with_capacity(vertices_num);
        for i in 0..vertices_num {
            vertices.push(Vertex {
                position: na::Vector3::new(
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                ),
                normal: na::Vector3::new(
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ),
                texture_coords: na::Vector2::new(mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]),
            })
        }

        // Handle material of mesh
        if let Some(material_id) = mesh.material_id {
            let material = &materials[material_id];

            let shininess = material.shininess;
            let mut diffuse_texture = None;
            let mut specular_texture = None;
            let mut normal_texture = None;

            // load diffuse map
            if let Some(ref diffuse_texture_filename) = material.diffuse_texture {
                diffuse_texture =
                    Some(self.load_texture(diffuse_texture_filename, TextureType::BlinnDiffuse)?);
            } else {
                warn!("No diffuse texture for mesh in model({})!", model_name)
            }
            // load specular map
            if let Some(ref specular_texture_filename) = material.specular_texture {
                specular_texture =
                    Some(self.load_texture(specular_texture_filename, TextureType::BlinnSpecular)?);
            } else {
                warn!("No specular texture for mesh in model({})!", model_name)
            }
            // load normal map
            if let Some(ref normal_texture_filename) = material.normal_texture {
                normal_texture =
                    Some(self.load_texture(normal_texture_filename, TextureType::Normal)?);
            } else {
                warn!("No normal texture for mesh in model({})!", model_name)
            }
            // TODO: load ambient & shiness map

            Ok(Mesh::new(
                vertices,
                indices,
                diffuse_texture,
                specular_texture,
                normal_texture,
                shininess,
            )?)
        } else {
            bail!("No material id for mesh")
        }
    }

    fn load_texture(
        &mut self,
        filename: &str,
        texture_type: TextureType,
    ) -> anyhow::Result<Texture> {
        let texture_path = self
            .model_path
            .parent()
            .unwrap_or(&PathBuf::new())
            .join(filename);

        // Check if texture was loaded before and if so, return it
        if let Some(texture_loaded) = self.textures_loaded.iter().find(|t| t.path == texture_path) {
            return Ok(texture_loaded.clone());
        }

        let texture = Texture::create(texture_path, Some(texture_type))?;
        self.textures_loaded.push(texture.clone());

        Ok(texture)
    }
}
