use anyhow::Ok;

#[allow(unused_variables)]
fn main() -> anyhow::Result<()> {
    let (models, materials) = tobj::load_obj(
        "assets/models/nanosuit/nanosuit.obj",
        &tobj::LoadOptions::default(),
    )?;

    Ok(())
}
