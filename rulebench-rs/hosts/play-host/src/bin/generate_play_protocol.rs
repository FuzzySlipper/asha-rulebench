use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let check = arguments.iter().any(|argument| argument == "--check");
    let output = arguments
        .iter()
        .find(|argument| argument.as_str() != "--check")
        .map(PathBuf::from)
        .ok_or("usage: generate_play_protocol [--check] <output>")?;
    let generated = rulebench_play_host::generated_protocol();

    if check {
        let existing = fs::read_to_string(&output)?;
        if existing != generated {
            return Err(format!("generated protocol is stale: {}", output.display()).into());
        }
        println!("generated play protocol is current");
        return Ok(());
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, generated)?;
    println!("generated {}", output.display());
    Ok(())
}
