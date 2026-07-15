use std::fs;
use std::path::PathBuf;

use rulebench_process_host::{build_durable_rulebench_router, render_capability_manifest_artifact};

fn main() {
    let root = temporary_artifact_root();
    let result = emit(&root);
    let _ = fs::remove_dir_all(&root);
    match result {
        Ok(rendered) => print!("{rendered}"),
        Err(error) => {
            eprintln!("capability manifest generation failed: {error}");
            std::process::exit(1);
        }
    }
}

fn emit(root: &PathBuf) -> Result<String, String> {
    let router = build_durable_rulebench_router(root)?;
    render_capability_manifest_artifact(router.capability_manifest())
        .map_err(|error| error.to_string())
}

fn temporary_artifact_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "asha-rulebench-capability-generation-{}",
        std::process::id()
    ))
}
