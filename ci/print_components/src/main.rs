use anyhow::Result;
use component::Components;

fn main() -> Result<()> {
    let components = Components::collect()?;

    let excluded_components = ["forc-lsp", "forc-fmt", "forc-explore", "forc-client"];

    for component in components.component.keys() {
        if !excluded_components.contains(&components.component[component].name.as_str()) {
            println!("{}", components.component[component].name);
        }
    }

    Ok(())
}
