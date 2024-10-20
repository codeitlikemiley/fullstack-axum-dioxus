use ructe::{Result, Ructe};

fn main() -> Result<()> {
    let mut ructe = Ructe::from_env().unwrap();
    let mut statics = ructe.statics().unwrap();
    statics.add_files("images").unwrap();
    ructe.compile_templates("images").unwrap();

    Ok(())
}
