Project Peril!:
===============
An attempt to learn Rust and Vulkan.  
Will probably result in a game or so, we'll see.

HowTos:
=======
## Prerequisites:
Get glslangValidator from https://cvs.khronos.org/svn/repos/ogl/trunk/ecosystem/public/sdk/tools/glslang/Install/  
Add it to your PATH and check that it works as intended:

~~~bash
$ glslangValidator -v
Glslang Version: SPIRV99.947 15-Feb-2016
ESSL Version: OpenGL ES GLSL 3.00 glslang LunarG Khronos.SPIRV99.947 15-Feb-2016
GLSL Version: 4.20 glslang LunarG Khronos.SPIRV99.947 15-Feb-2016
SPIR-V Version 0x00010000, Revision 6
GLSL.std.450 Version 100, Revision 1
Khronos Tool ID 8
~~~

Compilation of the glsl files is done automatically on regular compilation.  
For more details, check https://www.khronos.org/opengles/sdk/tools/Reference-Compiler/

## Compile:
~~~bash
$ cargo build [--release]
~~~

## Run
~~~bash
$ ./target/(debug|release)/project_peril
~~~

## Compile and run!:
~~~bash
$ cargo run [--release]
~~~

## Vulkan debug layer:
Add --features debug\_layer to your build line, like so:

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
Please create pull requests with reviewers for commits to the master branch. This is currently enforced by GitHub option.  
Please also use rustfmt on the code before opening code reviews. The project is currently using the nightly rustfmt,
which is used as follows:

~~~bash
$ rustup toolchain install nightly
$ rustup component add rustfmt --toolchain nightly
$ cargo +nightly fmt
~~~

For more information, see [rustfmt's GitHub page](https://github.com/rust-lang/rustfmt).
