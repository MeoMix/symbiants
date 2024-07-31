use fs_extra::dir::{self, CopyOptions};
use std::env;
use std::path::Path;

// Bevy expects the assets directory to exist adjacent to the executable
// When debugging Linux builds in VSCode, the executable is in the debug directory
// So, need to copy assets to the debug directory for them to be loaded properly.
fn main() {
    // Get the OUT_DIR environment variable where Cargo places build artifacts
    let out_dir = env::var("OUT_DIR").unwrap();

    // Define the source and destination paths
    let source_path = Path::new("assets");
    let dest_path = Path::new(&out_dir).join("../../../assets");

    // Create copy options
    let mut options = CopyOptions::new();
    options.overwrite = true;
    options.copy_inside = true;

    // Copy the assets folder to the target directory
    match dir::copy(source_path, dest_path, &options) {
        Ok(_) => println!("Assets copied successfully"),
        Err(e) => panic!("Failed to copy assets: {}", e),
    }
}
