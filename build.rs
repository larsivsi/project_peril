extern crate glob;

use glob::glob;
use std::path;
use std::process::Command;

fn compile_shader(shaderpath: path::PathBuf)
{
	let glsl_name = shaderpath.into_os_string().into_string().unwrap();
	let spv_name = glsl_name.replace(".", "_") + ".spv";

	let output = Command::new("glslangValidator")
		.args(&["-V", glsl_name.as_str(), "-o", spv_name.as_str()])
		.output()
		.expect("Could not execute glslangValidator, is it in PATH?");

	if !output.status.success()
	{
		panic!("Failed to build shader: {}", String::from_utf8_lossy(&output.stdout));
	}
}

fn main()
{
	// Only run if shaders have changed
	println!("cargo:rerun-if-changed=shaders");

	// Build shaders
	for vertpath in glob("shaders/*.vert").unwrap().filter_map(Result::ok)
	{
		compile_shader(vertpath);
	}

	for fragpath in glob("shaders/*.frag").unwrap().filter_map(Result::ok)
	{
		compile_shader(fragpath);
	}
}
