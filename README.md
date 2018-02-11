Project Peril!:
===============

An attempt to learn Rust and Vulkan.
Will probably result in a game or so, we'll see.

HowTos:
=======

## Compile:
$ cargo build [--release]

## Run:
$ ./target/(debug|release)/project\_peril

## Compile and run!:
$ cargo run [--release]

## To enable debug layer, specify --features debug\_layer in the cargo build command:
$ cargo run [--release] --features debug\_layer

## Compile glsl to spv:
Get glslangValidator from https://cvs.khronos.org/svn/repos/ogl/trunk/ecosystem/public/sdk/tools/glslang/Install/
Add it to your PATH
$ glslangValidator -V shader.<stage> [-o <output.spv>]
For more details, check https://www.khronos.org/opengles/sdk/tools/Reference-Compiler/
