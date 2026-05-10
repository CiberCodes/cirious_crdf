//! A parser for the OBJ file format, with support for materials and
//! triangulation.
use crate::config::Config;
use crate::models::{Face, Material, Mesh, Scene, Vector2, Vector3};
use std::io;

/// Represents errors that can occur during parsing.
#[derive(Debug)]
pub enum ParserError {
    /// An I/O error occurred while reading the file.
    Io(io::Error),
    /// The file contains invalid or malformed data.
    InvalidData(String),
}

impl From<io::Error> for ParserError {
    fn from(err: io::Error) -> Self {
        ParserError::Io(err)
    }
}

/// The main parser for OBJ files.
pub struct Parser<'a> {
    scene: Scene,
    config: &'a Config,
}

impl<'a> Parser<'a> {
    /// Creates a new `Parser` with the given configuration.
    pub fn new(config: &'a Config) -> Self {
        Self {
            scene: Scene::default(),
            config,
        }
    }

    /// Parses an OBJ file from a string.
    pub fn parse(mut self, content: &str) -> Result<Scene, ParserError> {
        let mut current_mesh: Option<Mesh> = None;
        let mut current_material_index: Option<u32> = None;

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "o" => {
                    if let Some(mesh) = current_mesh.take() {
                        self.scene.meshes.push(mesh);
                    }
                    let name = parts.get(1).unwrap_or(&"unnamed").to_string();
                    current_mesh = Some(Mesh {
                        name,
                        ..Default::default()
                    });
                }
                "v" => self.parse_vertex(&mut current_mesh, &parts, line_num)?,
                "vn" => self.parse_normal(&mut current_mesh, &parts, line_num)?,
                "vt" => self.parse_uv(&mut current_mesh, &parts, line_num)?,
                "f" => {
                    self.parse_face(&mut current_mesh, &parts, current_material_index, line_num)?
                }
                "p" => {
                    let parent_name = parts.get(1).map(|s| s.to_string());
                    if let Some(mesh) = current_mesh.as_mut() {
                        mesh.parent = parent_name;
                    }
                }
                "mtllib" => self.parse_mtllib(&parts, line_num)?,
                "usemtl" => {
                    current_material_index = self.parse_usemtl(&parts, line_num)?;
                }
                "m" => {
                    let path = parts.get(1).unwrap_or(&"").to_string();
                    if !path.is_empty() {
                        self.scene.external_models.push(path);
                    }
                }
                _ => { /* Ignore comments and other unsupported lines */ }
            }
        }

        if let Some(mesh) = current_mesh.take() {
            self.scene.meshes.push(mesh);
        }

        Ok(self.scene)
    }

    fn parse_vertex(
        &self,
        mesh: &mut Option<Mesh>,
        parts: &[&str],
        line_num: usize,
    ) -> Result<(), ParserError> {
        let mesh = mesh.as_mut().ok_or_else(|| {
            ParserError::InvalidData(format!(
                "Vertex found outside of an object definition at line {}",
                line_num
            ))
        })?;
        let scale = self.config.vertex_scale;
        let x = parts
            .get(1)
            .and_then(|s| s.parse::<f32>().ok())
            .ok_or_else(|| {
                ParserError::InvalidData(format!("Invalid vertex data at line {}", line_num))
            })?
            * scale;
        let y = parts
            .get(2)
            .and_then(|s| s.parse::<f32>().ok())
            .ok_or_else(|| {
                ParserError::InvalidData(format!("Invalid vertex data at line {}", line_num))
            })?
            * scale;
        let z = parts
            .get(3)
            .and_then(|s| s.parse::<f32>().ok())
            .ok_or_else(|| {
                ParserError::InvalidData(format!("Invalid vertex data at line {}", line_num))
            })?
            * scale;
        mesh.vertices.push(Vector3 { x, y, z });
        Ok(())
    }

    fn parse_normal(
        &self,
        mesh: &mut Option<Mesh>,
        parts: &[&str],
        line_num: usize,
    ) -> Result<(), ParserError> {
        let mesh = mesh.as_mut().ok_or_else(|| {
            ParserError::InvalidData(format!(
                "Normal found outside of an object definition at line {}",
                line_num
            ))
        })?;
        let x = parts.get(1).and_then(|s| s.parse().ok()).ok_or_else(|| {
            ParserError::InvalidData(format!("Invalid normal data at line {}", line_num))
        })?;
        let y = parts.get(2).and_then(|s| s.parse().ok()).ok_or_else(|| {
            ParserError::InvalidData(format!("Invalid normal data at line {}", line_num))
        })?;
        let z = parts.get(3).and_then(|s| s.parse().ok()).ok_or_else(|| {
            ParserError::InvalidData(format!("Invalid normal data at line {}", line_num))
        })?;
        mesh.normals.push(Vector3 { x, y, z });
        Ok(())
    }

    fn parse_uv(
        &self,
        mesh: &mut Option<Mesh>,
        parts: &[&str],
        line_num: usize,
    ) -> Result<(), ParserError> {
        let mesh = mesh.as_mut().ok_or_else(|| {
            ParserError::InvalidData(format!(
                "UV found outside of an object definition at line {}",
                line_num
            ))
        })?;
        let u = parts.get(1).and_then(|s| s.parse().ok()).ok_or_else(|| {
            ParserError::InvalidData(format!("Invalid UV data at line {}", line_num))
        })?;
        let v = parts.get(2).and_then(|s| s.parse().ok()).ok_or_else(|| {
            ParserError::InvalidData(format!("Invalid UV data at line {}", line_num))
        })?;
        mesh.uvs.push(Vector2 { u, v });
        Ok(())
    }

    fn parse_face(
        &self,
        mesh: &mut Option<Mesh>,
        parts: &[&str],
        material_index: Option<u32>,
        line_num: usize,
    ) -> Result<(), ParserError> {
        let mesh = mesh.as_mut().ok_or_else(|| {
            ParserError::InvalidData(format!(
                "Face found outside of an object definition at line {}",
                line_num
            ))
        })?;
        let mut face = Face {
            material_index,
            ..Default::default()
        };

        for part in &parts[1..] {
            let indices: Vec<Option<u32>> = part
                .split('/')
                .map(|s| s.parse().ok().map(|i: u32| i.saturating_sub(1)))
                .collect();

            face.vertex_indices
                .push(indices.get(0).and_then(|&x| x).ok_or_else(|| {
                    ParserError::InvalidData(format!(
                        "Invalid face vertex index at line {}",
                        line_num
                    ))
                })?);
            if let Some(Some(uv_idx)) = indices.get(1) {
                face.uv_indices.push(*uv_idx);
            }
            if let Some(Some(n_idx)) = indices.get(2) {
                face.normal_indices.push(*n_idx);
            }
        }

        if self.config.triangulate && face.vertex_indices.len() > 3 {
            mesh.faces.extend(self.triangulate_face(&face));
        } else {
            mesh.faces.push(face);
        }
        Ok(())
    }

    fn parse_mtllib(&mut self, parts: &[&str], line_num: usize) -> Result<(), ParserError> {
        let _lib_name = parts.get(1).ok_or_else(|| {
            ParserError::InvalidData(format!("Missing mtllib name at line {}", line_num))
        })?;
        // In a real implementation, parse the material library file.
        // For now, we'll create a dummy material.
        self.scene.materials.push(Material::default());
        Ok(())
    }

    fn parse_usemtl(&self, parts: &[&str], line_num: usize) -> Result<Option<u32>, ParserError> {
        let name = parts.get(1).ok_or_else(|| {
            ParserError::InvalidData(format!("Missing usemtl name at line {}", line_num))
        })?;
        Ok(self
            .scene
            .materials
            .iter()
            .position(|m| &m.name == name)
            .map(|i| i as u32))
    }

    fn triangulate_face(&self, face: &Face) -> Vec<Face> {
        let mut triangulated_faces = Vec::new();
        if face.vertex_indices.len() < 3 {
            return triangulated_faces;
        }

        let fan_start_v = face.vertex_indices[0];
        let fan_start_n = face.normal_indices.get(0).copied();
        let fan_start_uv = face.uv_indices.get(0).copied();

        for i in 1..face.vertex_indices.len() - 1 {
            let mut new_face = Face {
                vertex_indices: vec![
                    fan_start_v,
                    face.vertex_indices[i],
                    face.vertex_indices[i + 1],
                ],
                material_index: face.material_index,
                ..Default::default()
            };

            if let Some(n1) = fan_start_n {
                if let (Some(n2), Some(n3)) =
                    (face.normal_indices.get(i), face.normal_indices.get(i + 1))
                {
                    new_face.normal_indices.extend_from_slice(&[n1, *n2, *n3]);
                }
            }

            if let Some(uv1) = fan_start_uv {
                if let (Some(uv2), Some(uv3)) = (face.uv_indices.get(i), face.uv_indices.get(i + 1))
                {
                    new_face.uv_indices.extend_from_slice(&[uv1, *uv2, *uv3]);
                }
            }

            triangulated_faces.push(new_face);
        }

        triangulated_faces
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_obj() {
        let config = Config::default();
        let content = "
o MyObject
v 1.0 1.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
";
        let scene = Parser::new(&config).parse(content).unwrap();
        assert_eq!(scene.meshes.len(), 1);
        let mesh = &scene.meshes[0];
        assert_eq!(mesh.name, "MyObject");
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.faces.len(), 1);
        assert_eq!(mesh.faces[0].vertex_indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_parse_parent_tag() {
        let config = Config::default();
        let content = "
o Parent
v 0 0 0
o Child
p Parent
v 1 1 1
";
        let scene = Parser::new(&config).parse(content).unwrap();
        assert_eq!(scene.meshes.len(), 2);
        assert_eq!(scene.meshes[1].name, "Child");
        assert_eq!(scene.meshes[1].parent, Some("Parent".to_string()));
    }

    #[test]
    fn test_parse_invalid_data() {
        let config = Config::default();
        let content = "v 1.0 1.0"; // Missing z coordinate
        let result = Parser::new(&config).parse(content);
        assert!(matches!(result, Err(ParserError::InvalidData(_))));
    }
}
