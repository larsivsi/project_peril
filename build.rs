extern crate glob;

use glob::glob;
use std::env;
use std::process::Command;

fn main()
{
	// Get CWD
	let mut path = env::current_dir().unwrap();

	// Change path to the shaders-directory and generate shaders
	path.push("shaders");
	env::set_current_dir(&path).unwrap();

	// Build shaders
	for shader in glob("*.vert").unwrap()
	{
		let glsl_name = match shader
		{
			Ok(s) => s.into_os_string().into_string().unwrap(),
			Err(e) => panic!("{:?}", e),
		};
		let spv_name = glsl_name.replace(".", "_") + ".spv";

		println!("cargo:warning={} -> {}", glsl_name, spv_name);

		let output = Command::new("glslangValidator")
			.args(&["-V", glsl_name.as_str(), "-o", spv_name.as_str()])
			.output()
			.expect("Could not execute glslangValidator, is it in PATH?");

		if !output.status.success()
		{
			panic!("Failed to build shader: {}", String::from_utf8_lossy(&output.stdout));
		}

		println!("cargo:rerun-if-changed={}", glsl_name);
		println!("cargo:rerun-if-changed={}", spv_name);
	}

	for shader in glob("*.frag").unwrap()
	{
		let glsl_name = match shader
		{
			Ok(s) => s.into_os_string().into_string().unwrap(),
			Err(e) => panic!("{:?}", e),
		};
		let spv_name = glsl_name.replace(".", "_") + ".spv";

		println!("cargo:warning={} -> {}", glsl_name, spv_name);

		let output = Command::new("glslangValidator")
			.args(&["-V", glsl_name.as_str(), "-o", spv_name.as_str()])
			.output()
			.expect("Could not execute glslangValidator, is it in PATH?");

		if !output.status.success()
		{
			panic!("Failed to build shader: {}", String::from_utf8_lossy(&output.stdout));
		}

		println!("cargo:rerun-if-changed={}", glsl_name);
		println!("cargo:rerun-if-changed={}", spv_name);
	}

	// Return to first directory
	path.pop();
	env::set_current_dir(&path).unwrap();
}
