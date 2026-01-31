use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use dtd_viewer::output;
use dtd_viewer::parser::parse_dtd;

#[derive(Parser)]
#[command(name = "dtd-viewer", about = "Visualize DTD file structure")]
struct Cli {
    /// DTD file to display
    file: PathBuf,

    /// Static tree output (non-interactive)
    #[arg(long = "static")]
    static_mode: bool,

    /// JSON output
    #[arg(long)]
    json: bool,

    /// Mermaid diagram output
    #[arg(long)]
    mermaid: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let input = std::fs::read_to_string(&cli.file)?;
    let dtd = parse_dtd(&input)?;

    if cli.json {
        output::json::print_json(&dtd)?;
    } else if cli.mermaid {
        output::mermaid::print_mermaid(&dtd);
    } else if cli.static_mode || !atty::is(atty::Stream::Stdout) {
        output::static_tree::print_static(&dtd);
    } else {
        dtd_viewer::tui::run(dtd)?;
    }

    Ok(())
}
