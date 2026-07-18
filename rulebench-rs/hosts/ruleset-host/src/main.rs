use std::env;
use std::fs;

use rulebench_ruleset_host::{RulesetDiagnosticDto, RulesetHost, RulesetWorkspaceResponseDto};
use tiny_http::{Header, Method, Response, Server, StatusCode};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let options = Options::parse()?;
    let prepared_source = fs::read(&options.prepared_path)?;
    let host = RulesetHost::new(prepared_source);
    let server = Server::http(&options.address)?;
    println!("RULESET_HOST_URL=http://{}", options.address);

    for request in server.incoming_requests() {
        let method = request.method().clone();
        let url = request.url().to_owned();
        let (status, payload) = route(&host, &method, &url);
        let encoded = serde_json::to_vec(&payload)?;
        let content_type = Header::from_bytes("content-type", "application/json")
            .map_err(|()| "invalid constant content-type header")?;
        request.respond(
            Response::from_data(encoded)
                .with_status_code(StatusCode(status))
                .with_header(content_type),
        )?;
    }
    Ok(())
}

fn route(host: &RulesetHost, method: &Method, url: &str) -> (u16, RulesetWorkspaceResponseDto) {
    match (method, url) {
        (&Method::Get, "/api/ruleset") | (&Method::Get, "/api/ruleset/health") => {
            (200, host.status())
        }
        (&Method::Post, "/api/ruleset/compile") => {
            let response = host.compile_candidate();
            (if response.ok { 200 } else { 422 }, response)
        }
        (&Method::Post, "/api/ruleset/activate") => {
            let response = host.activate_candidate();
            (if response.ok { 200 } else { 409 }, response)
        }
        _ => {
            let mut response = host.status();
            response.ok = false;
            response.diagnostics = vec![RulesetDiagnosticDto {
                stage: "transport".to_owned(),
                severity: "error".to_owned(),
                code: "RULESET_ROUTE_NOT_FOUND".to_owned(),
                path: url.to_owned(),
                message: format!("unsupported request {method} {url}"),
            }];
            (404, response)
        }
    }
}

struct Options {
    address: String,
    prepared_path: String,
}

impl Options {
    fn parse() -> Result<Self, String> {
        let arguments = env::args().skip(1).collect::<Vec<_>>();
        let address = argument_value(&arguments, "--address")?;
        let prepared_path = argument_value(&arguments, "--prepared")?;
        Ok(Self {
            address,
            prepared_path,
        })
    }
}

fn argument_value(arguments: &[String], name: &str) -> Result<String, String> {
    let index = arguments
        .iter()
        .position(|argument| argument == name)
        .ok_or_else(|| format!("missing {name}"))?;
    arguments
        .get(index + 1)
        .cloned()
        .ok_or_else(|| format!("missing value for {name}"))
}
