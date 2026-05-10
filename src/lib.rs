#![doc = include_str!("../README.md")]
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

pub mod config;
pub mod models;
pub mod parser;

use crate::config::Config;
pub use crate::models::*;
use crate::parser::{Parser, ParserError};
use serde_json;

#[derive(Debug)]
pub enum CrdError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Parser(ParserError),
    InvalidIndex(String),
    UnknownFileType,
}

impl From<std::io::Error> for CrdError {
    fn from(err: std::io::Error) -> Self {
        CrdError::Io(err)
    }
}

impl From<serde_json::Error> for CrdError {
    fn from(err: serde_json::Error) -> Self {
        CrdError::Json(err)
    }
}

impl From<ParserError> for CrdError {
    fn from(err: ParserError) -> Self {
        CrdError::Parser(err)
    }
}

pub fn new() -> Scene {
    Scene::default()
}

pub fn clone(scene: &Scene) -> Scene {
    scene.clone()
}

pub fn add_mesh(scene: &mut Scene, mesh: Mesh) {
    scene.meshes.push(mesh);
}

pub fn remove_mesh(scene: &mut Scene, index: usize) {
    if index < scene.meshes.len() {
        scene.meshes.remove(index);
    }
}

pub fn add_material(scene: &mut Scene, material: Material) {
    scene.materials.push(material);
}

pub fn remove_material(scene: &mut Scene, index: usize) {
    if index < scene.materials.len() {
        scene.materials.remove(index);
        for mesh in &mut scene.meshes {
            for face in &mut mesh.faces {
                if let Some(mat_idx) = face.material_index {
                    if mat_idx == index as u32 {
                        face.material_index = None;
                    } else if mat_idx > index as u32 {
                        face.material_index = Some(mat_idx - 1);
                    }
                }
            }
        }
    }
}

pub fn assign_material(scene: &mut Scene, mesh_index: usize, material_index: usize) {
    if let Some(mesh) = scene.meshes.get_mut(mesh_index) {
        if material_index < scene.materials.len() {
            for face in &mut mesh.faces {
                face.material_index = Some(material_index as u32);
            }
        }
    }
}

pub fn clear_materials(scene: &mut Scene) {
    scene.materials.clear();
    for mesh in &mut scene.meshes {
        for face in &mut mesh.faces {
            face.material_index = None;
        }
    }
}

pub fn get_mesh(scene: &Scene, index: usize) -> Option<&Mesh> {
    scene.meshes.get(index)
}

pub fn get_material(scene: &Scene, index: usize) -> Option<&Material> {
    scene.materials.get(index)
}

pub fn get_mesh_mut(scene: &mut Scene, index: usize) -> Option<&mut Mesh> {
    scene.meshes.get_mut(index)
}

pub fn get_material_mut(scene: &mut Scene, index: usize) -> Option<&mut Material> {
    scene.materials.get_mut(index)
}

pub fn update_mesh(scene: &mut Scene, index: usize, mesh: Mesh) {
    if let Some(m) = scene.meshes.get_mut(index) {
        *m = mesh;
    }
}

pub fn update_material(scene: &mut Scene, index: usize, material: Material) {
    if let Some(m) = scene.materials.get_mut(index) {
        *m = material;
    }
}

pub fn get_mesh_by_name<'a>(scene: &'a Scene, name: &str) -> Option<&'a Mesh> {
    scene.meshes.iter().find(|m| m.name == name)
}

pub fn get_material_by_name<'a>(scene: &'a Scene, name: &str) -> Option<&'a Material> {
    scene.materials.iter().find(|m| m.name == name)
}

pub fn get_mesh_by_name_mut<'a>(scene: &'a mut Scene, name: &str) -> Option<&'a mut Mesh> {
    scene.meshes.iter_mut().find(|m| m.name == name)
}

pub fn get_material_by_name_mut<'a>(scene: &'a mut Scene, name: &str) -> Option<&'a mut Material> {
    scene.materials.iter_mut().find(|m| m.name == name)
}

pub fn get_meshes(scene: &Scene) -> &Vec<Mesh> {
    &scene.meshes
}

pub fn get_meshes_mut(scene: &mut Scene) -> &mut Vec<Mesh> {
    &mut scene.meshes
}

pub fn get_materials(scene: &Scene) -> &Vec<Material> {
    &scene.materials
}

pub fn get_materials_mut(scene: &mut Scene) -> &mut Vec<Material> {
    &mut scene.materials
}

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Scene, CrdError> {
    let file_type = get_file_type(path.as_ref()).ok_or(CrdError::UnknownFileType)?;
    match file_type {
        FileType::CRDF => Ok(load_crdf(path, &Config::default())?),
        FileType::Obj => Ok(import_from_obj(path)?),
        FileType::Json => {
            let content = fs::read_to_string(path)?;
            Ok(from_json(&content)?)
        }
        FileType::Unknown => Err(CrdError::UnknownFileType),
    }
}

pub fn to_file<P: AsRef<Path>>(path: P, scene: &Scene) -> Result<(), CrdError> {
    let file_type = get_file_type(path.as_ref()).ok_or(CrdError::UnknownFileType)?;
    match file_type {
        FileType::CRDF => Ok(save_crdf(path, scene, &Config::default())?),
        FileType::Obj => Ok(export_to_obj(path, scene)?),
        FileType::Json => {
            let content = to_json(scene)?;
            Ok(fs::write(path, content)?)
        }
        FileType::Unknown => Err(CrdError::UnknownFileType),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    CRDF,
    Obj,
    Json,
    Unknown,
}

pub fn get_file_type<P: AsRef<Path>>(path: P) -> Option<FileType> {
    path.as_ref()
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| match s.to_lowercase().as_str() {
            "crdf" => FileType::CRDF,
            "scene" => FileType::CRDF,
            "cirious" => FileType::CRDF,
            "obj" => FileType::Obj,
            "json" => FileType::Json,
            _ => FileType::Unknown,
        })
}

pub fn load_crdf<P: AsRef<Path>>(path: P, config: &Config) -> Result<Scene, ParserError> {
    let content = fs::read_to_string(path)?;
    let parser = Parser::new(config);
    parser.parse(&content)
}

pub fn save_crdf<P: AsRef<Path>>(_path: P, _scene: &Scene, _config: &Config) -> std::io::Result<()> {
    // This is a placeholder for the future CRDF format.
    Ok(())
}

pub fn export_to_obj<P: AsRef<Path>>(path: P, scene: &Scene) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    for material in &scene.materials {
        writeln!(file, "newmtl {}", material.name)?;
        writeln!(file, "Ka {} {} {}", material.ambient_color[0], material.ambient_color[1], material.ambient_color[2])?;
        writeln!(file, "Kd {} {} {}", material.diffuse_color[0], material.diffuse_color[1], material.diffuse_color[2])?;
        writeln!(file, "Ks {} {} {}", material.specular_color[0], material.specular_color[1], material.specular_color[2])?;
        writeln!(file, "Ns {}", material.shininess)?;
        if let Some(texture) = &material.diffuse_texture {
            writeln!(file, "map_Kd {}", texture)?;
        }
    }
    for mesh in &scene.meshes {
        writeln!(file, "o {}", mesh.name)?;
        for vertex in &mesh.vertices {
            writeln!(file, "v {} {} {}", vertex.x, vertex.y, vertex.z)?;
        }
        for normal in &mesh.normals {
            writeln!(file, "vn {} {} {}", normal.x, normal.y, normal.z)?;
        }
        for uv in &mesh.uvs {
            writeln!(file, "vt {} {}", uv.u, uv.v)?;
        }
        let mut last_material_index = None;
        for face in &mesh.faces {
            if face.material_index != last_material_index {
                if let Some(index) = face.material_index {
                    if let Some(material) = scene.materials.get(index as usize) {
                        writeln!(file, "usemtl {}", material.name)?;
                    }
                }
                last_material_index = face.material_index;
            }
            write!(file, "f")?;
            for i in 0..face.vertex_indices.len() {
                let v_idx = face.vertex_indices[i] + 1;
                let uv_idx = face.uv_indices.get(i).map_or("".to_string(), |&id| (id + 1).to_string());
                let n_idx = face.normal_indices.get(i).map_or("".to_string(), |&id| (id + 1).to_string());
                write!(file, " {}/{}/{}", v_idx, uv_idx, n_idx)?;
            }
            writeln!(file)?;
        }
    }
    Ok(())
}

pub fn validate(scene: &Scene) -> Result<(), CrdError> {
    for mesh in &scene.meshes {
        for face in &mesh.faces {
            for &v_idx in &face.vertex_indices {
                if v_idx as usize >= mesh.vertices.len() {
                    return Err(CrdError::InvalidIndex(format!("Invalid vertex index: {}", v_idx)));
                }
            }
            for &n_idx in &face.normal_indices {
                if n_idx as usize >= mesh.normals.len() {
                    return Err(CrdError::InvalidIndex(format!("Invalid normal index: {}", n_idx)));
                }
            }
            for &uv_idx in &face.uv_indices {
                if uv_idx as usize >= mesh.uvs.len() {
                    return Err(CrdError::InvalidIndex(format!("Invalid uv index: {}", uv_idx)));
                }
            }
            if let Some(mat_idx) = face.material_index {
                if mat_idx as usize >= scene.materials.len() {
                    return Err(CrdError::InvalidIndex(format!("Invalid material index: {}", mat_idx)));
                }
            }
        }
    }
    Ok(())
}

pub fn optimize(scene: &mut Scene) {
    for mesh in &mut scene.meshes {
        let mut new_mesh = Mesh {
            name: mesh.name.clone(),
            ..Default::default()
        };
        let mut vertex_map = HashMap::new();
        let mut normal_map = HashMap::new();
        let mut uv_map = HashMap::new();
        for face in &mesh.faces {
            let mut new_face = Face {
                material_index: face.material_index,
                ..Default::default()
            };
            for i in 0..face.vertex_indices.len() {
                let v_idx = face.vertex_indices[i] as usize;
                let vertex = mesh.vertices[v_idx];
                let key = ((vertex.x * 1000.0) as i32, (vertex.y * 1000.0) as i32, (vertex.z * 1000.0) as i32);
                let new_v_idx = *vertex_map.entry(key).or_insert_with(|| {
                    new_mesh.vertices.push(vertex);
                    new_mesh.vertices.len() as u32 - 1
                });
                new_face.vertex_indices.push(new_v_idx);
                if let Some(&n_idx) = face.normal_indices.get(i) {
                    let normal = mesh.normals[n_idx as usize];
                    let key = ((normal.x * 1000.0) as i32, (normal.y * 1000.0) as i32, (normal.z * 1000.0) as i32);
                    let new_n_idx = *normal_map.entry(key).or_insert_with(|| {
                        new_mesh.normals.push(normal);
                        new_mesh.normals.len() as u32 - 1
                    });
                    new_face.normal_indices.push(new_n_idx);
                }
                if let Some(&uv_idx) = face.uv_indices.get(i) {
                    let uv = mesh.uvs[uv_idx as usize];
                    let key = ((uv.u * 1000.0) as i32, (uv.v * 1000.0) as i32);
                    let new_uv_idx = *uv_map.entry(key).or_insert_with(|| {
                        new_mesh.uvs.push(uv);
                        new_mesh.uvs.len() as u32 - 1
                    });
                    new_face.uv_indices.push(new_uv_idx);
                }
            }
            new_mesh.faces.push(new_face);
        }
        *mesh = new_mesh;
    }
}

pub fn weld(scene: &mut Scene, tolerance: f32) {
    for mesh in &mut scene.meshes {
        let mut vertex_map = HashMap::new();
        let mut new_vertices = Vec::new();
        let mut old_to_new_indices = vec![0; mesh.vertices.len()];
        for (i, vertex) in mesh.vertices.iter().enumerate() {
            let key = (
                (vertex.x / tolerance).round() as i32,
                (vertex.y / tolerance).round() as i32,
                (vertex.z / tolerance).round() as i32,
            );
            let new_idx = *vertex_map.entry(key).or_insert_with(|| {
                new_vertices.push(*vertex);
                new_vertices.len() as u32 - 1
            });
            old_to_new_indices[i] = new_idx;
        }
        mesh.vertices = new_vertices;
        for face in &mut mesh.faces {
            for v_idx in &mut face.vertex_indices {
                *v_idx = old_to_new_indices[*v_idx as usize];
            }
        }
    }
}

pub fn unweld(scene: &mut Scene) {
    for mesh in &mut scene.meshes {
        let mut new_mesh = Mesh {
            name: mesh.name.clone(),
            ..Default::default()
        };
        for face in &mesh.faces {
            let mut new_face = Face {
                material_index: face.material_index,
                ..Default::default()
            };
            for i in 0..face.vertex_indices.len() {
                let v_idx = face.vertex_indices[i] as usize;
                new_mesh.vertices.push(mesh.vertices[v_idx]);
                new_face.vertex_indices.push(new_mesh.vertices.len() as u32 - 1);
                if let Some(&n_idx) = face.normal_indices.get(i) {
                    new_mesh.normals.push(mesh.normals[n_idx as usize]);
                    new_face.normal_indices.push(new_mesh.normals.len() as u32 - 1);
                }
                if let Some(&uv_idx) = face.uv_indices.get(i) {
                    new_mesh.uvs.push(mesh.uvs[uv_idx as usize]);
                    new_face.uv_indices.push(new_mesh.uvs.len() as u32 - 1);
                }
            }
            new_mesh.faces.push(new_face);
        }
        *mesh = new_mesh;
    }
}

pub fn transform(scene: &mut Scene, matrix: &[[f32; 4]; 4]) {
    for mesh in &mut scene.meshes {
        for vertex in &mut mesh.vertices {
            let x = vertex.x * matrix[0][0] + vertex.y * matrix[1][0] + vertex.z * matrix[2][0] + matrix[3][0];
            let y = vertex.x * matrix[0][1] + vertex.y * matrix[1][1] + vertex.z * matrix[2][1] + matrix[3][1];
            let z = vertex.x * matrix[0][2] + vertex.y * matrix[1][2] + vertex.z * matrix[2][2] + matrix[3][2];
            vertex.x = x;
            vertex.y = y;
            vertex.z = z;
        }
        for normal in &mut mesh.normals {
            let x = normal.x * matrix[0][0] + normal.y * matrix[1][0] + normal.z * matrix[2][0];
            let y = normal.x * matrix[0][1] + normal.y * matrix[1][1] + normal.z * matrix[2][1];
            let z = normal.x * matrix[0][2] + normal.y * matrix[1][2] + normal.z * matrix[2][2];
            *normal = Vector3::new(x, y, z).normalize();
        }
    }
}

pub fn center(scene: &mut Scene) {
    let (min, max) = get_bounding_box(scene);
    let offset = Vector3 {
        x: (min.x + max.x) / 2.0,
        y: (min.y + max.y) / 2.0,
        z: (min.z + max.z) / 2.0,
    };
    for mesh in &mut scene.meshes {
        for vertex in &mut mesh.vertices {
            *vertex = *vertex - offset;
        }
    }
}

pub fn merge(scene: &mut Scene, new_mesh_name: &str) {
    if scene.meshes.len() <= 1 {
        return;
    }
    let mut merged_mesh = Mesh {
        name: new_mesh_name.to_string(),
        ..Default::default()
    };
    for mesh in &scene.meshes {
        let v_offset = merged_mesh.vertices.len() as u32;
        let n_offset = merged_mesh.normals.len() as u32;
        let uv_offset = merged_mesh.uvs.len() as u32;
        merged_mesh.vertices.extend_from_slice(&mesh.vertices);
        merged_mesh.normals.extend_from_slice(&mesh.normals);
        merged_mesh.uvs.extend_from_slice(&mesh.uvs);
        for face in &mesh.faces {
            let mut new_face = face.clone();
            for v_idx in &mut new_face.vertex_indices {
                *v_idx += v_offset;
            }
            for n_idx in &mut new_face.normal_indices {
                *n_idx += n_offset;
            }
            for uv_idx in &mut new_face.uv_indices {
                *uv_idx += uv_offset;
            }
            merged_mesh.faces.push(new_face);
        }
    }
    scene.meshes = vec![merged_mesh];
}

pub fn split(scene: &mut Scene) {
    let mut new_meshes = Vec::new();
    for mesh in &scene.meshes {
        if mesh.faces.is_empty() {
            new_meshes.push(mesh.clone());
            continue;
        }
        let mut face_sets = Vec::new();
        let mut visited_faces = vec![false; mesh.faces.len()];
        for i in 0..mesh.faces.len() {
            if visited_faces[i] {
                continue;
            }
            let mut current_set = Vec::new();
            let mut q = std::collections::VecDeque::new();
            q.push_back(i);
            visited_faces[i] = true;
            while let Some(face_idx) = q.pop_front() {
                current_set.push(face_idx);
                for j in 0..mesh.faces.len() {
                    if !visited_faces[j] && faces_are_connected(mesh, face_idx, j) {
                        q.push_back(j);
                        visited_faces[j] = true;
                    }
                }
            }
            face_sets.push(current_set);
        }
        if face_sets.len() <= 1 {
            new_meshes.push(mesh.clone());
            continue;
        }
        for (i, face_indices) in face_sets.into_iter().enumerate() {
            let mut sub_mesh = Mesh {
                name: format!("{}_part{}", mesh.name, i),
                ..Default::default()
            };
            let mut old_to_new_v = HashMap::new();
            let mut old_to_new_n = HashMap::new();
            let mut old_to_new_uv = HashMap::new();
            for face_idx in face_indices {
                let face = &mesh.faces[face_idx];
                let mut new_face = face.clone();
                for i in 0..face.vertex_indices.len() {
                    let old_v_idx = face.vertex_indices[i];
                    let new_v_idx = *old_to_new_v.entry(old_v_idx).or_insert_with(|| {
                        sub_mesh.vertices.push(mesh.vertices[old_v_idx as usize]);
                        sub_mesh.vertices.len() as u32 - 1
                    });
                    new_face.vertex_indices[i] = new_v_idx;
                    if let Some(&old_n_idx) = face.normal_indices.get(i) {
                        let new_n_idx = *old_to_new_n.entry(old_n_idx).or_insert_with(|| {
                            sub_mesh.normals.push(mesh.normals[old_n_idx as usize]);
                            sub_mesh.normals.len() as u32 - 1
                        });
                        new_face.normal_indices[i] = new_n_idx;
                    }
                    if let Some(&old_uv_idx) = face.uv_indices.get(i) {
                        let new_uv_idx = *old_to_new_uv.entry(old_uv_idx).or_insert_with(|| {
                            sub_mesh.uvs.push(mesh.uvs[old_uv_idx as usize]);
                            sub_mesh.uvs.len() as u32 - 1
                        });
                        new_face.uv_indices[i] = new_uv_idx;
                    }
                }
                sub_mesh.faces.push(new_face);
            }
            new_meshes.push(sub_mesh);
        }
    }
    scene.meshes = new_meshes;
}

fn faces_are_connected(mesh: &Mesh, face1_idx: usize, face2_idx: usize) -> bool {
    let face1 = &mesh.faces[face1_idx];
    let face2 = &mesh.faces[face2_idx];
    for v1 in &face1.vertex_indices {
        for v2 in &face2.vertex_indices {
            if v1 == v2 {
                return true;
            }
        }
    }
    false
}

pub fn flip_faces(scene: &mut Scene) {
    for mesh in &mut scene.meshes {
        for face in &mut mesh.faces {
            face.vertex_indices.reverse();
            face.normal_indices.reverse();
            face.uv_indices.reverse();
        }
    }
}

pub fn flip_normals(scene: &mut Scene) {
    for mesh in &mut scene.meshes {
        for normal in &mut mesh.normals {
            *normal = *normal * -1.0;
        }
    }
}

pub fn calculate_normals(scene: &mut Scene) {
    for mesh in &mut scene.meshes {
        mesh.normals.clear();
        let mut face_normals = Vec::new();
        for face in &mesh.faces {
            if face.vertex_indices.len() < 3 {
                face_normals.push(Vector3::new(0.0, 0.0, 1.0));
                continue;
            }
            let v0 = mesh.vertices[face.vertex_indices[0] as usize];
            let v1 = mesh.vertices[face.vertex_indices[1] as usize];
            let v2 = mesh.vertices[face.vertex_indices[2] as usize];
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            face_normals.push(edge1.cross(edge2).normalize());
        }
        let mut vertex_normals: Vec<Vec<Vector3>> = vec![Vec::new(); mesh.vertices.len()];
        for (i, face) in mesh.faces.iter().enumerate() {
            for &v_idx in &face.vertex_indices {
                vertex_normals[v_idx as usize].push(face_normals[i]);
            }
        }
        for normals in vertex_normals.iter_mut() {
            let mut avg_normal = Vector3::default();
            for normal in normals.iter() {
                avg_normal = avg_normal + *normal;
            }
            mesh.normals.push(avg_normal.normalize());
        }
        for face in &mut mesh.faces {
            face.normal_indices = face.vertex_indices.clone();
        }
    }
}

pub fn smooth_normals(scene: &mut Scene, angle_threshold: f32) {
    for mesh in &mut scene.meshes {
        let mut vertex_normals: Vec<Vec<Vector3>> = vec![Vec::new(); mesh.vertices.len()];
        for face in &mesh.faces {
            if face.vertex_indices.len() < 3 {
                continue;
            }
            let v0 = mesh.vertices[face.vertex_indices[0] as usize];
            let v1 = mesh.vertices[face.vertex_indices[1] as usize];
            let v2 = mesh.vertices[face.vertex_indices[2] as usize];
            let face_normal = (v1 - v0).cross(v2 - v0);
            for &v_idx in &face.vertex_indices {
                vertex_normals[v_idx as usize].push(face_normal);
            }
        }
        let mut new_normals = Vec::new();
        let mut old_to_new_n = HashMap::new();
        for (v_idx, face_normals) in vertex_normals.iter().enumerate() {
            let mut smooth_normal = Vector3::default();
            for normal1 in face_normals {
                let mut group_normal = *normal1;
                for normal2 in face_normals {
                    if normal1.dot(*normal2).acos() < angle_threshold.to_radians() {
                        group_normal = group_normal + *normal2;
                    }
                }
                smooth_normal = group_normal.normalize();
                break;
            }
            let key = ((smooth_normal.x * 1000.0) as i32, (smooth_normal.y * 1000.0) as i32, (smooth_normal.z * 1000.0) as i32);
            let new_n_idx = *old_to_new_n.entry(key).or_insert_with(|| {
                new_normals.push(smooth_normal);
                new_normals.len() as u32 - 1
            });
            for face in &mut mesh.faces {
                for i in 0..face.vertex_indices.len() {
                    if face.vertex_indices[i] as usize == v_idx {
                        if i < face.normal_indices.len() {
                            face.normal_indices[i] = new_n_idx;
                        } else {
                            face.normal_indices.push(new_n_idx);
                        }
                    }
                }
            }
        }
        mesh.normals = new_normals;
    }
}

pub fn subdivide(_scene: &mut Scene, _levels: u32) {
    // Placeholder for Catmull-Clark subdivision
}

pub fn decimate(_scene: &mut Scene, _target_face_count: u32) {
    // Placeholder for mesh decimation
}

pub fn import_from_obj<P: AsRef<Path>>(path: P) -> Result<Scene, ParserError> {
    let content = fs::read_to_string(path)?;
    let config = Config::default();
    let parser = Parser::new(&config);
    parser.parse(&content)
}

pub fn get_bounding_box(scene: &Scene) -> (Vector3, Vector3) {
    let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
    for mesh in &scene.meshes {
        for vertex in &mesh.vertices {
            min.x = min.x.min(vertex.x);
            min.y = min.y.min(vertex.y);
            min.z = min.z.min(vertex.z);
            max.x = max.x.max(vertex.x);
            max.y = max.y.max(vertex.y);
            max.z = max.z.max(vertex.z);
        }
    }
    (min, max)
}

pub fn get_scene_info(scene: &Scene) -> String {
    let (vertices, faces, triangles) = get_scene_size(scene);
    format!(
        "Scene Information:\n  Meshes: {}\n  Materials: {}\n  Vertices: {}\n  Faces: {}\n  Triangles: {}",
        scene.meshes.len(),
        scene.materials.len(),
        vertices,
        faces,
        triangles
    )
}

pub fn to_json(scene: &Scene) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(scene)
}

pub fn from_json(json: &str) -> Result<Scene, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn get_textures(scene: &Scene) -> Vec<String> {
    scene.materials.iter().filter_map(|m| m.diffuse_texture.clone()).collect()
}

pub fn get_meshes_by_material(scene: &Scene, material_index: usize) -> Vec<&Mesh> {
    scene
        .meshes
        .iter()
        .filter(|m| m.faces.iter().any(|f| f.material_index == Some(material_index as u32)))
        .collect()
}

pub fn get_faces_by_material(mesh: &Mesh, material_index: usize) -> Vec<&Face> {
    mesh.faces
        .iter()
        .filter(|f| f.material_index == Some(material_index as u32))
        .collect()
}

pub fn get_mesh_names(scene: &Scene) -> Vec<String> {
    scene.meshes.iter().map(|m| m.name.clone()).collect()
}

pub fn get_material_names(scene: &Scene) -> Vec<String> {
    scene.materials.iter().map(|m| m.name.clone()).collect()
}

pub fn get_texture_names(scene: &Scene) -> Vec<String> {
    scene.materials.iter().filter_map(|m| m.diffuse_texture.clone()).collect()
}

pub fn get_scene_size(scene: &Scene) -> (usize, usize, usize) {
    let mut vertices = 0;
    let mut faces = 0;
    let mut triangles = 0;
    for mesh in &scene.meshes {
        vertices += mesh.vertices.len();
        faces += mesh.faces.len();
        for face in &mesh.faces {
            if face.vertex_indices.len() >= 3 {
                triangles += face.vertex_indices.len() - 2;
            }
        }
    }
    (vertices, faces, triangles)
}

pub fn get_mesh_by_face(scene: &Scene, face_index: usize) -> Option<&Mesh> {
    let mut count = 0;
    for mesh in &scene.meshes {
        if face_index < count + mesh.faces.len() {
            return Some(mesh);
        }
        count += mesh.faces.len();
    }
    None
}

pub fn get_material_by_face<'a>(scene: &'a Scene, mesh: &Mesh, face_index: usize) -> Option<&'a Material> {
    if let Some(face) = mesh.faces.get(face_index) {
        if let Some(material_index) = face.material_index {
            return scene.materials.get(material_index as usize);
        }
    }
    None
}

pub fn get_faces_by_vertex(mesh: &Mesh, vertex_index: usize) -> Vec<&Face> {
    mesh.faces
        .iter()
        .filter(|f| f.vertex_indices.contains(&(vertex_index as u32)))
        .collect()
}

pub fn get_face_count(scene: &Scene) -> usize {
    scene.meshes.iter().map(|m| m.faces.len()).sum()
}

pub fn get_vertex_count(scene: &Scene) -> usize {
    scene.meshes.iter().map(|m| m.vertices.len()).sum()
}

pub fn get_triangle_count(scene: &Scene) -> usize {
    scene.meshes.iter().map(|m| {
        m.faces.iter().map(|f| {
            if f.vertex_indices.len() >= 3 {
                f.vertex_indices.len() - 2
            } else {
                0
            }
        }).sum::<usize>()
    }).sum()
}

pub fn get_material_count(scene: &Scene) -> usize {
    scene.materials.len()
}

pub fn get_texture_count(scene: &Scene) -> usize {
    scene.materials.iter().filter(|m| m.diffuse_texture.is_some()).count()
}

pub fn get_mesh_count(scene: &Scene) -> usize {
    scene.meshes.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Mesh, Vector3};

    #[test]
    fn test_add_and_remove_mesh() {
        let mut scene = new();
        let mesh = Mesh::default();
        add_mesh(&mut scene, mesh);
        assert_eq!(get_mesh_count(&scene), 1);
        remove_mesh(&mut scene, 0);
        assert_eq!(get_mesh_count(&scene), 0);
    }

    #[test]
    fn test_center_scene() {
        let mut scene = new();
        let mut mesh = Mesh::default();
        mesh.vertices.push(Vector3::new(10.0, 10.0, 10.0));
        mesh.vertices.push(Vector3::new(20.0, 20.0, 20.0));
        add_mesh(&mut scene, mesh);

        center(&mut scene);

        let (min, max) = get_bounding_box(&scene);
        assert!((min.x + max.x).abs() < 1e-6);
        assert!((min.y + max.y).abs() < 1e-6);
        assert!((min.z + max.z).abs() < 1e-6);
    }

    #[test]
    fn test_flip_normals() {
        let mut scene = new();
        let mut mesh = Mesh::default();
        mesh.normals.push(Vector3::new(1.0, 0.0, 0.0));
        add_mesh(&mut scene, mesh);

        flip_normals(&mut scene);

        assert_eq!(scene.meshes[0].normals[0], Vector3::new(-1.0, 0.0, 0.0));
    }
}
