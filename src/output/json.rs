use crate::model::Dtd;

pub fn print_json(dtd: &Dtd) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(dtd)?;
    println!("{}", json);
    Ok(())
}
