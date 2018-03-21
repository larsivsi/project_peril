use std::env;
use std::process::Command;

fn main()
{
	// Get CWD
	let mut path = env::current_dir().unwrap();

	// Change path to the shaders-directory and generate shaders
	path.push("shaders");
	env::set_current_dir(&path).unwrap();
	let output = Command::new("./generate_spv.sh").output().expect("Failed to generate shaders");
	println!("cargo:rerun-if-changed={}", path.display());
	println!("{}", output.status);
	println!("{}", String::from_utf8_lossy(&output.stdout));

	// Return to first directory
	path.pop();
	env::set_current_dir(&path).unwrap();
}
