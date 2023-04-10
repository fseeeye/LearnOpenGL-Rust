# LearnOpenGL-Rust

My OpenGL Learning code in Rust, depends on OpenGL3.3 & GLFW.

## Pre-requisities

* install [Rust](https://www.rust-lang.org/tools/install)
* install [cmake](https://cmake.org/download/) in your system for "glfw-rs" crate.

## Get Started

* Run OpenGL examples bellow by: `cargo run --example <xxx>`

### Examples

description for all examples:

1. [**Draw Triangle**](https://github.com/fseeeye/LearnOpenGL-Rust/tree/main/examples/001-draw-triangle): Draw a solid color triangle on window.
2. [**Draw Quad**](https://github.com/fseeeye/LearnOpenGL-Rust/tree/main/examples/002-draw-quad): Draw a solid color quad on window.
3. [**Texture**](https://github.com/fseeeye/LearnOpenGL-Rust/tree/main/examples/003-texture): Apply texture on quad simply.
4. [**Transform**](https://github.com/fseeeye/LearnOpenGL-Rust/tree/main/examples/004-transform): Apply MVP Transform.
5. [**Depth Test**](https://github.com/fseeeye/LearnOpenGL-Rust/tree/main/examples/005-depth-test): Apply Depth Test to show multiple cubes.
6. [**Camera**](https://github.com/fseeeye/LearnOpenGL-Rust/tree/main/examples/006-camera): Impl a camera.

## Dependencies

* [gl](https://crates.io/crates/gl) : OpenGL bindings.
* [glfw](https://crates.io/crates/glfw) : Window - C++ GLFW3 bindings and idiomatic wrapper.
* [nalgebra](https://crates.io/crates/nalgebra) : General-purpose linear algebra library with transformations and statically-sized or dynamically-sized matrices.
* [bytemuck](https://crates.io/crates/bytemuck) : bit cast between data types
* [image](https://crates.io/crates/image) : basic image processing functions and methods for converting to and from various image formats.
* [tracing](https://crates.io/crates/tracing) : logger.

## Ref

* [LearnOpenGL](https://learnopengl.com/)
* [LearnOpenGl - Rust](https://rust-tutorials.github.io/learn-opengl/)
