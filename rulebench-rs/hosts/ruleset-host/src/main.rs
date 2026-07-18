use std::env;

use rulebench_ruleset_host::{
    PreparedRulesetCompileRequestDto, RulesetDiagnosticDto, RulesetHost,
    RulesetWorkspaceResponseDto,
};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let options = Options::parse()?;
    let host = RulesetHost::new();
    let server = Server::http(&options.address)?;
    println!("RULESET_HOST_URL=http://{}", options.address);

    for mut request in server.incoming_requests() {
        let method = request.method().clone();
        let url = request.url().to_owned();
        let (status, payload) = route(&host, &method, &url, &mut request);
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

fn route(
    host: &RulesetHost,
    method: &Method,
    url: &str,
    request: &mut Request,
) -> (u16, RulesetWorkspaceResponseDto) {
    match (method, url) {
        (&Method::Get, "/api/ruleset") | (&Method::Get, "/api/ruleset/health") => {
            (200, host.status())
        }
        (&Method::Post, "/api/ruleset/compile") => match decode_compile_request(request) {
            Ok(compile_request) => {
                let response = host.compile_candidate(&compile_request.prepared_source);
                (if response.ok { 200 } else { 422 }, response)
            }
            Err(diagnostic) => {
                let mut response = host.status();
                response.ok = false;
                response.diagnostics = vec![*diagnostic];
                (400, response)
            }
        },
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
                package_id: None,
                definition_id: None,
                source: None,
                graph_path: None,
                expected: None,
                actual: None,
            }];
            (404, response)
        }
    }
}

fn decode_compile_request(
    request: &mut Request,
) -> Result<PreparedRulesetCompileRequestDto, Box<RulesetDiagnosticDto>> {
    let mut body = String::new();
    request
        .as_reader()
        .read_to_string(&mut body)
        .map_err(|error| Box::new(request_diagnostic(error.to_string())))?;
    serde_json::from_str(&body).map_err(|error| Box::new(request_diagnostic(error.to_string())))
}

fn request_diagnostic(message: String) -> RulesetDiagnosticDto {
    RulesetDiagnosticDto {
        stage: "transport".to_owned(),
        severity: "error".to_owned(),
        code: "RULESET_COMPILE_REQUEST_INVALID".to_owned(),
        path: "$".to_owned(),
        message,
        package_id: None,
        definition_id: None,
        source: None,
        graph_path: None,
        expected: None,
        actual: None,
    }
}

struct Options {
    address: String,
}

impl Options {
    fn parse() -> Result<Self, String> {
        let arguments = env::args().skip(1).collect::<Vec<_>>();
        let address = argument_value(&arguments, "--address")?;
        Ok(Self { address })
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
