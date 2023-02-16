# LearnOpenGL-Rust
My OpenGL Learning code in Rust, depends on OpenGL3.3 & GLFW.

## Pre-requisities

* install [Rust](https://www.rust-lang.org/tools/install)
* install [cmake](https://cmake.org/download/) in your system for "glfw-rs" crate.

## Get Started

```bash
$ cargo run --example <xxx>
```

## OpenGL Examples

description for all examples:
1. Draw Triangle: draw a solid color triangle on window.
2. Abstract: how to wrap some OpenGL API into safe funcs.

## Dependencies
* [gl](https://crates.io/crates/gl) : OpenGL bindings.
* [glfw](https://crates.io/crates/glfw) : Window - C++ GLFW3 bindings and idiomatic wrapper.
* [nalgebra](https://crates.io/crates/nalgebra) : General-purpose linear algebra library with transformations and statically-sized or dynamically-sized matrices.
* [bytemuck](https://crates.io/crates/bytemuck) : bit cast between data types
* [image](https://crates.io/crates/image) : basic image processing functions and methods for converting to and from various image formats.

## Ref
* [Learn OpenGl Rust](https://rust-tutorials.github.io/learn-opengl/)
