# LearnOpenGL-Rust

My OpenGL Learning code in Rust, depends on OpenGL3.3 & GLFW.

## Pre-requisities

* install [Rust](https://www.rust-lang.org/tools/install)
* install [cmake](https://cmake.org/download/) in your system for "glfw-rs" crate.

## Get Started

* Run OpenGL examples bellow by: `cargo run --example <xxx>`

### Examples

description for all examples.

foundation:
1. [**Draw Triangle**](examples/foundation/001_draw_triangle.rs): Draw a solid color triangle on window.
2. [**Draw Quad**](examples/foundation/002_draw_quad.rs): Draw a solid color quad on window.
3. [**Texture**](examples/foundation/003_texture.rs): Apply texture on quad simply.
4. [**Transform**](examples/foundation/004_transform.rs): Apply MVP Transform.
5. [**Depth Test**](examples/foundation/005_depth_test.rs): Apply Depth Test to show multiple cubes.
6. [**Camera**](examples/foundation/006_camera.rs): Impl a camera.

lighting:
1. [**Simple Color**](examples/lighting/007_simple_color.rs): Create a simplest light source.
2. [**Blinn-Phong**](examples/lighting/008_blinn_phong.rs): Blinn-Phong Model.
3. [**Material Map**](examples/lighting/009_material_map.rs): Abstract simplest Blinn-Phong material.

## Core Dependencies

* [gl](https://crates.io/crates/gl) : OpenGL bindings.
* [glfw](https://crates.io/crates/glfw) : Window - C++ GLFW3 bindings and idiomatic wrapper.
* [nalgebra](https://crates.io/crates/nalgebra) : General-purpose linear algebra library with transformations and statically-sized or dynamically-sized matrices.
* [bytemuck](https://crates.io/crates/bytemuck) : bit cast between data types
* [image](https://crates.io/crates/image) : basic image processing functions and methods for converting to and from various image formats.
* [tracing](https://crates.io/crates/tracing) : logger.

## Ref

* [LearnOpenGL](https://learnopengl.com/)
* [LearnOpenGl - Rust](https://rust-tutorials.github.io/learn-opengl/)
