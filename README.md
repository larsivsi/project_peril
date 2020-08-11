Project Peril!:
===============
An attempt to learn Rust and Vulkan.  
Will probably result in a game or so, we'll see.

HowTos:
=======

Prerequisites:
--------------
### glslangValidator
The glslangValidator is used for compiling [GLSL into SPIR-V](https://www.khronos.org/spir/).  
For more details, check [Khronos' OpenGL ES reference compiler webpage](https://www.khronos.org/opengles/sdk/tools/Reference-Compiler/).

glslangValidator can be installed using your favorite package manager:
~~~bash
$ sudo apt install glslang-tools
~~~

Once installed, check that the binary is available in your PATH:
~~~bash
$ glslangValidator -v
Glslang Version: 7.11.3214
ESSL Version: OpenGL ES GLSL 3.20 glslang Khronos. 11.3214
GLSL Version: 4.60 glslang Khronos. 11.3214
SPIR-V Version 0x00010300, Revision 7
GLSL.std.450 Version 100, Revision 1
Khronos Tool ID 8
SPIR-V Generator Version 7
GL_KHR_vulkan_glsl version 100
ARB_GL_gl_spirv version 100
~~~

Compilation of the [glsl files in this project](shaders) is done automatically when building (see [build.rs](build.rs)).

### Vulkan validation layers
Though not strictly required, it's highly recommended to use the validation layers during development to find bugs.

Again, use your favorite package manager to install them:
~~~bash
$ sudo apt install vulkan-validationlayers vulkan-tools
~~~

Then verify that they are available to your vulkan instance:
~~~bash
$ vulkaninfo | grep VK_LAYER_KHRONOS_validation
VK_LAYER_KHRONOS_validation (Khronos Validation Layer) Vulkan version 1.2.141, layer version 1:
~~~

### SDL2
This project uses the SDL2 Rust crate in "bundled" mode. It therefore requires a C-compiler and cmake installed:
~~~
$ sudo apt install gcc cmake
~~~
In addition, the development headers for the current window system need to be available.  
For X11, this requires:
~~~
$ sudo apt install libx11-dev libxext-dev
~~~
See [the rust-sdl2 github page](https://github.com/Rust-SDL2/rust-sdl2) for more details.

Compile:
--------
~~~bash
$ cargo build [--release]
~~~

Run:
----
~~~bash
$ ./target/(debug|release)/project_peril
~~~

Compile and run!:
-----------------
~~~bash
$ cargo run [--release]
~~~

Vulkan debug layer:
-------------------
Add --features debug\_layer to your build/run line, like so:
~~~bash
$ cargo run [--release] --features debug_layer
~~~

License:
========
The code in this project is licensed under [MIT license](LICENSE).  
The original assets of this project are licensed under [CC BY 4.0](assets/original/LICENSE), unless otherwise stated.  
[Third party assets](assets/thirdparty/) will have their respective license alongside the asset files.

Contribute:
===========
Please create pull requests with reviewers for commits to the main branch. This is currently enforced by GitHub option.  
Please also use rustfmt on the code before opening code reviews. The project is currently using the nightly rustfmt,
which is used as follows:

~~~bash
$ rustup toolchain install nightly
$ rustup component add rustfmt --toolchain nightly
$ cargo +nightly fmt
~~~

For more information, see [rustfmt's GitHub page](https://github.com/rust-lang/rustfmt).
