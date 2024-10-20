use ructe::{Result, Ructe};

fn main() -> Result<()> {
    let mut ructe = Ructe::from_env().unwrap();
    let mut statics = ructe.statics().unwrap();
    statics.add_files("images").unwrap();
    statics
        .add_file_as("./js/pages/users/components_bg.wasm", "components_bg.wasm")
        .unwrap();
    statics
        .add_file_as("./js/pages/users/components.js", "components.js")
        .unwrap();
    ructe.compile_templates("images").unwrap();

    Ok(())
}
