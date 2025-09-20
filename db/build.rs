use std::env;
use std::ffi::OsString;
use std::path::Path;

fn main() {
    // Compile our SQL
    cornucopia();
}

fn cornucopia() {
    // Check if DATABASE_URL is set
    if env::var_os("DATABASE_URL").is_none() {
        println!("cargo:warning=DATABASE_URL not set, creating empty cornucopia.rs");
        println!("cargo:warning=Set DATABASE_URL if you need to generate database code");

        // Create empty file to avoid compilation errors
        // Handle case where OUT_DIR might not be available (like in rust analyzer)
        let out_dir = env::var_os("OUT_DIR")
            .unwrap_or_else(|| {
                println!("cargo:warning=OUT_DIR not set, using current directory");
                OsString::from(".")
            });

        let file_path = Path::new(&out_dir).join("cornucopia.rs");

        // Create the directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
        }

        std::fs::write(&file_path, "").unwrap();
        return;
    }

    // For the sake of simplicity, this example uses the defaults.
    let queries_path = "queries";

    let out_dir = env::var_os("OUT_DIR")
        .unwrap_or_else(|| {
            println!("cargo:warning=OUT_DIR not set, using current directory");
            OsString::from(".")
        });

    let file_path = Path::new(&out_dir).join("cornucopia.rs");
    let db_url = env::var_os("DATABASE_URL").unwrap();

    // Rerun this build script if the queries or migrations change.
    println!("cargo:rerun-if-changed={queries_path}");

    // Call cornucopia. Use whatever CLI command you need.
    let output = std::process::Command::new("cornucopia")
        .arg("-q")
        .arg(queries_path)
        .arg("--serialize")
        .arg("-d")
        .arg(&file_path)
        .arg("live")
        .arg(db_url)
        .output()
        .unwrap();

    // If Cornucopia couldn't run properly, try to display the error.
    if !output.status.success() {
        panic!("{}", &std::str::from_utf8(&output.stderr).unwrap());
    }
}