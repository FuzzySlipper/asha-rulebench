use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use rulebench_process_host::{build_rulebench_bridge, serve_until, ProcessHostRouter};

fn main() {
    if let Err(error) = run() {
        eprintln!("rulebench process host failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let bind_address = bind_address()?;
    let listener = TcpListener::bind(&bind_address)?;
    let local_address = listener.local_addr()?;
    let bridge = build_rulebench_bridge()?;
    let router = ProcessHostRouter::new(bridge);
    println!("RULEBENCH_HOST_URL=http://{local_address}");
    serve_until(listener, router, Arc::new(AtomicBool::new(false)))?;
    Ok(())
}

fn bind_address() -> Result<String, String> {
    let mut args = std::env::args().skip(1);
    let mut bind = "127.0.0.1:4318".to_string();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--bind" => {
                bind = args
                    .next()
                    .ok_or_else(|| "--bind requires an address".to_string())?;
            }
            "--help" | "-h" => {
                println!("Usage: rulebench_process_host [--bind 127.0.0.1:4318]");
                std::process::exit(0);
            }
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }
    Ok(bind)
}
