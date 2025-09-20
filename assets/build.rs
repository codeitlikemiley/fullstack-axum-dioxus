use ructe::{Result, Ructe};
use std::fs;

fn main() -> Result<()> {
    let mut ructe = Ructe::from_env()?;
    let mut statics = ructe.statics()?;
    statics.add_files("images")?;

    let pages_dir = "./js/pages";

    let skip_files = [".gitignore", "package.json"];

    println!("cargo:warning=Searching for files in: {}", pages_dir);
    add_generated_wasm_files(&mut statics, pages_dir, &skip_files)?;

    ructe.compile_templates("images")?;

    println!("Template compilation complete.");

    Ok(())
}

fn add_generated_wasm_files(
    statics: &mut ructe::StaticFiles,
    dir: &str,
    skip_files: &[&str],
) -> Result<()> {
    // Check if directory exists before trying to read it
    match fs::metadata(dir) {
        Ok(_) => {} // Directory exists, continue
        Err(_) => {
            println!("cargo:warning=Directory '{}' does not exist, skipping", dir);
            return Ok(());
        }
    }

    let entries = fs::read_dir(dir).map_err(ructe::RucteError::from)?;

    for entry in entries {
        match entry {
            Ok(dir_entry) => {
                let path = dir_entry.path();
                let filename = match path.file_name() {
                    Some(name) => name.to_str().unwrap(),
                    None => continue,
                };

                println!("cargo:warning=Current file being processed: {}", filename);
                println!("cargo:warning=Full path: {:?}", path.display());

                if path.is_dir() {
                    println!("cargo:warning=Entering directory: {}", filename);
                    // Recursively process the subdirectory
                    add_generated_wasm_files(statics, path.to_str().unwrap(), skip_files)?;
                } else if skip_files.contains(&filename) {
                    println!("cargo:warning=Skipping file: {}", filename);
                    continue;
                } else if let Some(extension) = path.extension() {
                    let relative_path = path.display().to_string();
                    println!("cargo:warning=relative path: {}", relative_path);
                    match extension.to_str() {
                        Some("wasm") | Some("ts") | Some("js") => {
                            println!(
                                "cargo:warning=Adding file: {} as {}",
                                relative_path, filename
                            );
                            if let Err(e) = statics.add_file_as(&relative_path, filename) {
                                println!("cargo:warning=Failed to add file {}: {:?}", filename, e);
                            }
                        }
                        _ => {
                            println!("cargo:warning=Skipping unsupported file type: {}", filename);
                        }
                    }
                }
            }
            Err(e) => {
                println!("cargo:warning=Error reading directory entry: {:?}", e);
            }
        }
    }
    Ok(())
}
