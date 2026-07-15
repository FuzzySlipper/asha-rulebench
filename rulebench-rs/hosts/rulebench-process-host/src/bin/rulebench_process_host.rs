use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use std::path::PathBuf;

use rulebench_process_host::{
    build_durable_rulebench_router, build_rulebench_bridge, serve_until, ProcessHostRouter,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("rulebench process host failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = options()?;
    let bind_address = options.bind;
    let listener = TcpListener::bind(&bind_address)?;
    let local_address = listener.local_addr()?;
    let router = match options.artifact_root {
        Some(root) => build_durable_rulebench_router(&root)?,
        None => ProcessHostRouter::new(build_rulebench_bridge()?),
    };
    let repository = router.repository_status();
    println!(
        "RULEBENCH_ARTIFACT_REPOSITORY={} content={} replays={}",
        repository.mode, repository.content_artifact_count, repository.replay_artifact_count
    );
    for issue in &repository.issues {
        eprintln!(
            "RULEBENCH_ARTIFACT_ISSUE kind={} code={} path={} message={}",
            issue.artifact_kind, issue.code, issue.path, issue.message
        );
    }
    println!("RULEBENCH_HOST_URL=http://{local_address}");
    serve_until(listener, router, Arc::new(AtomicBool::new(false)))?;
    Ok(())
}

struct HostOptions {
    bind: String,
    artifact_root: Option<PathBuf>,
}

fn options() -> Result<HostOptions, String> {
    let mut args = std::env::args().skip(1);
    let mut bind = "127.0.0.1:4318".to_string();
    let mut artifact_root = std::env::var_os("RULEBENCH_ARTIFACT_ROOT").map(PathBuf::from);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--bind" => {
                bind = args
                    .next()
                    .ok_or_else(|| "--bind requires an address".to_string())?;
            }
            "--artifact-root" => {
                artifact_root =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "--artifact-root requires a path".to_string()
                    })?));
            }
            "--help" | "-h" => {
                println!(
                    "Usage: rulebench_process_host [--bind 127.0.0.1:4318] [--artifact-root PATH]"
                );
                std::process::exit(0);
            }
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }
    Ok(HostOptions {
        bind,
        artifact_root,
    })
}
