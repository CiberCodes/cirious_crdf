# Cirious CRDF

Cirious CRDF is a Rust library for working with 3D model data. It provides a set of tools for loading, saving, and manipulating 3D scenes, with a focus on the OBJ file format. The library is designed to be modular and extensible, with a clear and well-documented API.

## Features

- **Load and Save 3D Models**: Load scenes from OBJ and JSON files, and export them back.
- **Scene Manipulation**: A comprehensive set of functions for manipulating scenes, including adding, removing, and updating meshes and materials.
- **Mesh Processing**: A variety of tools for processing meshes, such as welding, unwelding, splitting, merging, and calculating normals.
- **Geometric Transformations**: Apply transformations to meshes, including translation, rotation, and scaling.
- **Data Introspection**: A rich set of functions for querying information about scenes, meshes, materials, and more.
- **Configuration**: A flexible configuration system for customizing the behavior of the library.
- **Error Handling**: A unified error handling system for robust and predictable behavior.

## Getting Started

To use Cirious CRDF in your project, add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
cirious_crdf = { git = "https://github.com/CiberCodes/cirious_crdf" }
```

Then, you can start using the library in your code:

```rust
use cirious_crdf::{from_file, center, to_file, FileType};

fn main() {
    // Load a scene from an OBJ file
    let mut scene = from_file("my_model.obj").expect("Failed to load scene");

    // Center the scene at the origin
    center(&mut scene);

    // Save the modified scene to a new file
    to_file("my_model_centered.obj", &scene).expect("Failed to save scene");
}
```

## API Overview

The `cirious_crdf` library provides a wide range of functions for working with 3D scenes. Here are some of the most important ones:

- **`from_file(path)`**: Loads a scene from a file, automatically detecting the file type.
- **`to_file(path, scene)`**: Saves a scene to a file, automatically detecting the file type.
- **`new()`**: Creates a new, empty scene.
- **`add_mesh(scene, mesh)`**: Adds a mesh to a scene.
- **`remove_mesh(scene, index)`**: Removes a mesh from a scene by its index.
- **`center(scene)`**: Centers a scene at the origin.
- **`transform(scene, matrix)`**: Applies a transformation matrix to a scene.
- **`validate(scene)`**: Validates the integrity of a scene.

For a complete list of functions and their documentation, please refer to the source code.
