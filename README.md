# rust-wasm-3D

Project inspired by a Alan Ji's interest in programming 3D Rendering engines from scratch.
This project is built on top of Rust, WebAssembly, and and a buffer of u8's representing the canvas.

All the math needed to do projections was implemented with the help of 
- https://scratchapixel.com
- https://gabrielgambetta.com/computer-graphics-from-scratch/
- Javidx9 on YouTube

Fast Line Drawing Algorithm adapted from implementation of "Bresenham's line drawing algorithm" Wikipedia page
- https://en.wikipedia.org//wiki/Bresenham's_line_algorithm

TODO:
- Proper culling before 2D projection when both points in the line are outside the frustum
- Save the scene info to static memory, so that the scene is not regenerated from scratch every frame
- House the rendering engine functionality in a file outside lib.rs
