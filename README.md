# LearnOpenGL-Rust

My OpenGL Learning code in Rust, depends on OpenGL3.3.

## Pre-requisities

* install [Rust](https://www.rust-lang.org/tools/install)
* install [cmake](https://cmake.org/download/) in your system for "glfw-rs" crate.

## Get Started

* Run OpenGL examples bellow by: `cargo run --example <xxx>`

## Examples

description for all examples.

### foundation
1. [**Draw Triangle**](examples/foundation/001_draw_triangle.rs): Draw a solid color triangle on window.
2. [**Draw Quad**](examples/foundation/002_draw_quad.rs): Draw a solid color quad on window.
3. [**Texture**](examples/foundation/003_texture.rs): Apply texture on quad simply.
4. [**Transform**](examples/foundation/004_transform.rs): Apply MVP Transform.
5. [**Depth Test**](examples/foundation/005_depth_test.rs): Apply Depth Test to show multiple cubes.
6. [**Camera**](examples/foundation/006_camera.rs): Impl a camera.

### lighting

1. [**Simple Color**](examples/lighting/007_simple_color.rs): Create a simplest light source.
2. [**Blinn-Phong**](examples/lighting/008_blinn_phong.rs): Blinn-Phong Model.
3. [**Material Map**](examples/lighting/009_material_map.rs): Abstract simplest Blinn-Phong material.
4. [**Multiple Lights**](examples/lighting/010_multi_lights.rs): implement multiple types of light source.

### model loading

1. [**Model Loading**](examples/model_loading/011_model_loading.rs): Load OBJ model from file.

### advanced opengl

1. [**Advanced Depth Test**](examples/advanced_opengl/012_advanced_depth_test.rs)
2. [**Advanced Stencil Test**](examples/advanced_opengl/013_advanced_stencil_test.rs)
3. [**Discard**](examples/advanced_opengl/014_discard.rs)
4. [**Advanced Stencil Test**](examples/advanced_opengl/015_blending.rs)
5. [**Face Culling**](examples/advanced_opengl/016_face_culling.rs)
6. [**FrameBuffer**](examples/advanced_opengl/017_frame_buffer.rs)
7. [**CubeMap**](examples/advanced_opengl/018_cube_map.rs)

### advanced lighting

1. [**Shadow Map**](examples/advanced_lighting/019_shadow_mapping.rs)
2. [**Normal Map**](examples/advanced_lighting/020_normal_map.rs)
3. [**Parallax Map**](examples/advanced_lighting/021_parallax_map.rs)
4. [**Tone Mapping**](examples/advanced_lighting/022_tone_mapping.rs)
5. [**Bloom**](examples/advanced_lighting/023_bloom.rs)
6. [**Deferred Rendering**](examples/advanced_lighting/024_deferred_rendering.rs)
7. [**SSAO**](examples/advanced_lighting/025_ssao.rs)

## TODO

* [ ] add imgui.

## Core Dependencies

* [gl](https://crates.io/crates/gl) : OpenGL bindings.
* [glfw](https://crates.io/crates/glfw) : Window - C++ GLFW3 bindings and idiomatic wrapper.
* [nalgebra](https://crates.io/crates/nalgebra) : General-purpose linear algebra library with transformations and statically-sized or dynamically-sized matrices.
* [bytemuck](https://crates.io/crates/bytemuck) : bit cast between data types
* [image](https://crates.io/crates/image) : basic image processing functions and methods for converting to and from various image formats.
* [tracing](https://crates.io/crates/tracing) : logger.

## Ref

* [learnopengl](https://learnopengl.com/)
* [Learn OpenGL Rust](https://rust-tutorials.github.io/learn-opengl/)
* [learn-opengl-rs](https://github.com/bwasty/learn-opengl-rs/)
