//! Represents the core data structures for 3D scenes, including vectors,
//! meshes, materials, and faces.
use serde::{Serialize, Deserialize};
use std::ops::{Add, Sub, Mul, Div};

/// A 3-dimensional vector used for vertices, normals, and other geometric
/// calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Vector3 {
    /// The x component of the vector.
    pub x: f32,
    /// The y component of the vector.
    pub y: f32,
    /// The z component of the vector.
    pub z: f32,
}

impl Vector3 {
    /// Creates a new `Vector3`.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Calculates the dot product of two vectors.
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Calculates the cross-product of two vectors.
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Returns the size (length) of the vector.
    pub fn magnitude(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Returns a normalized version of the vector.
    pub fn normalize(self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            self / mag
        } else {
            self
        }
    }
}

impl Add for Vector3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vector3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Div<f32> for Vector3 {
    type Output = Self;
    fn div(self, scalar: f32) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

/// A 2-dimensional vector used for texture coordinates (UVs).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Vector2 {
    /// The u component of the vector.
    pub u: f32,
    /// The v component of the vector.
    pub v: f32,
}

/// Represents a single face in a mesh, with indices for vertices, normals,
/// and UVs, along with an optional material index.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Face {
    /// The indices of the vertices that make up this face.
    pub vertex_indices: Vec<u32>,
    /// The indices of the normals associated with each vertex in this face.
    pub normal_indices: Vec<u32>,
    /// The indices of the UV coordinates associated with each vertex in this face.
    pub uv_indices: Vec<u32>,
    /// The index of the material applied to this face.
    pub material_index: Option<u32>,
}

/// A mesh containing the geometric data for a 3D object.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Mesh {
    /// The name of the mesh.
    pub name: String,
    /// The name of the parent mesh, if any.
    pub parent: Option<String>,
    /// The vertices of the mesh.
    pub vertices: Vec<Vector3>,
    /// The normals of the mesh.
    pub normals: Vec<Vector3>,
    /// The UV coordinates of the mesh.
    pub uvs: Vec<Vector2>,
    /// The tangents of the mesh.
    pub tangents: Vec<Vector3>,
    /// The joint indices affecting each vertex (4 joints per vertex).
    pub joints: Vec<[u32; 4]>,
    /// The weights of each joint affecting each vertex (4 weights per vertex).
    pub weights: Vec<[f32; 4]>,
    /// The faces of the mesh.
    pub faces: Vec<Face>,
}

/// Represents a skeletal skin, mapping joints to inverse bind matrices.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Skin {
    /// The name of the skin.
    pub name: String,
    /// The indices of the joints in the scene's node list.
    pub joint_indices: Vec<usize>,
    /// Inverse bind matrices for each joint.
    pub inverse_bind_matrices: Vec<[[f32; 4]; 4]>,
}

/// Represents an animation channel property.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationProperty {
    /// Translation (Vector3).
    Translation,
    /// Rotation (Quaternion [x, y, z, w]).
    Rotation,
    /// Scale (Vector3).
    Scale,
}

/// Represents a single animation keyframe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    /// Time in seconds.
    pub time: f32,
    /// The value at this time (depends on property).
    pub value: Vec<f32>,
}

/// Represents an animation channel targeting a specific node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationChannel {
    /// The index of the target node in the scene.
    pub target_node_index: usize,
    /// The property being animated.
    pub property: AnimationProperty,
    /// The list of keyframes.
    pub keyframes: Vec<Keyframe>,
}

/// Represents a full animation.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Animation {
    /// The name of the animation.
    pub name: String,
    /// The channels in this animation.
    pub channels: Vec<AnimationChannel>,
}

/// Represents a material with color, shininess, and texture information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    /// The name of the material.
    pub name: String,
    /// The ambient color of the material.
    pub ambient_color: [f32; 3],
    /// The diffuse color of the material.
    pub diffuse_color: [f32; 3],
    /// The specular color of the material.
    pub specular_color: [f32; 3],
    /// The shininess of the material.
    pub shininess: f32,
    /// The metallic factor of the material (0.0 to 1.0).
    pub metallic_factor: f32,
    /// The roughness factor of the material (0.0 to 1.0).
    pub roughness_factor: f32,
    /// The diffuse texture of the material, if any.
    pub diffuse_texture: Option<String>,
    /// The metallic-roughness texture of the material, if any.
    pub metallic_roughness_texture: Option<String>,
    /// The normal texture of the material, if any.
    pub normal_texture: Option<String>,
    /// The specular texture of the material, if any.
    pub specular_texture: Option<String>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            ambient_color: [0.2, 0.2, 0.2],
            diffuse_color: [0.8, 0.8, 0.8],
            specular_color: [1.0, 1.0, 1.0],
            shininess: 64.0,
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            diffuse_texture: None,
            metallic_roughness_texture: None,
            normal_texture: None,
            specular_texture: None,
        }
    }
}

/// A node in a scene hierarchy.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Node {
    /// The name of the node.
    pub name: String,
    /// The translation of the node.
    pub translation: [f32; 3],
    /// The rotation of the node (quaternion).
    pub rotation: [f32; 4],
    /// The scale of the node.
    pub scale: [f32; 3],
    /// Indices of child nodes.
    pub children: Vec<usize>,
    /// Index of the mesh attached to this node.
    pub mesh_index: Option<usize>,
    /// Index of the skin attached to this node.
    pub skin_index: Option<usize>,
}

/// Represents a full scene, containing meshes, materials, skins, and animations.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Scene {
    /// The meshes in the scene.
    pub meshes: Vec<Mesh>,
    /// The materials in the scene.
    pub materials: Vec<Material>,
    /// The skins (skeletons) in the scene.
    pub skins: Vec<Skin>,
    /// The animations in the scene.
    pub animations: Vec<Animation>,
    /// The hierarchy of nodes in the scene.
    pub nodes: Vec<Node>,
    /// External model files to be loaded into the scene (e.g., glTF).
    pub external_models: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3_dot_product() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        assert_eq!(v1.dot(v2), 32.0);
    }
}
