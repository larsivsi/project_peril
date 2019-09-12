extern crate glob;

use glob::glob;
use std::env;
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
	let shaderdir = "shaders";

	// Only run if shaders have changed
	println!("cargo:rerun-if-changed={}", shaderdir);

	// Get CWD
	let mut path = env::current_dir().unwrap();

	// Change path to the shaders-directory and generate shaders
	path.push(shaderdir);
	env::set_current_dir(&path).unwrap();

	// Build shaders
	for vertpath in glob("*.vert").unwrap().filter_map(Result::ok)
	{
		compile_shader(vertpath);
	}

	for fragpath in glob("*.frag").unwrap().filter_map(Result::ok)
	{
		compile_shader(fragpath);
	}

	// Return to first directory
	path.pop();
	env::set_current_dir(&path).unwrap();
}
