use std::env;

use rulebench_play_host::{
    GameplayCommandRequestDto, GameplayReactionRequestDto, GameplayTurnControlRequestDto,
    PlayDiagnosticDto, PlayHost, PlayWorkspaceResponseDto, PreparedPlayBundleCompileRequestDto,
    ScenarioSetupRequestDto, ScriptedGameplayRandomSource,
};
use serde::de::DeserializeOwned;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let options = Options::parse()?;
    let host = match env::var("RULEBENCH_RANDOM_TAPE") {
        Ok(source) => PlayHost::with_random_source(ScriptedGameplayRandomSource::new(
            parse_random_tape(&source)?,
        )),
        Err(env::VarError::NotPresent) => PlayHost::new(),
        Err(error) => return Err(error.into()),
    };
    let server = Server::http(&options.address)?;
    println!("PLAY_HOST_URL=http://{}", options.address);

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

fn parse_random_tape(source: &str) -> Result<Vec<u32>, String> {
    if source.trim().is_empty() {
        return Err("RULEBENCH_RANDOM_TAPE must contain comma-separated die values".to_owned());
    }
    source
        .split(',')
        .enumerate()
        .map(|(index, value)| {
            value.trim().parse::<u32>().map_err(|error| {
                format!(
                    "RULEBENCH_RANDOM_TAPE value {} is invalid: {error}",
                    index + 1
                )
            })
        })
        .collect()
}

fn route(
    host: &PlayHost,
    method: &Method,
    url: &str,
    request: &mut Request,
) -> (u16, PlayWorkspaceResponseDto) {
    match (method, url) {
        (&Method::Get, "/api/play") | (&Method::Get, "/api/play/health") => (200, host.status()),
        (&Method::Post, "/api/play-bundle/compile") => match decode_compile_request(request) {
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
        (&Method::Post, "/api/play-bundle/activate") => {
            let response = host.activate_candidate();
            (if response.ok { 200 } else { 409 }, response)
        }
        (&Method::Post, "/api/scenario/start") => {
            match decode_request::<ScenarioSetupRequestDto>(request) {
                Ok(setup) => {
                    let response = host.start_encounter(setup);
                    (if response.ok { 200 } else { 422 }, response)
                }
                Err(diagnostic) => invalid_request(host, diagnostic),
            }
        }
        (&Method::Post, "/api/session/command") => {
            match decode_request::<GameplayCommandRequestDto>(request) {
                Ok(command) => {
                    let response = host.execute_command(command);
                    (200, response)
                }
                Err(diagnostic) => invalid_request(host, diagnostic),
            }
        }
        (&Method::Post, "/api/session/reaction") => {
            match decode_request::<GameplayReactionRequestDto>(request) {
                Ok(reaction) => {
                    let response = host.resolve_reaction(reaction);
                    (200, response)
                }
                Err(diagnostic) => invalid_request(host, diagnostic),
            }
        }
        (&Method::Post, "/api/session/control") => {
            match decode_request::<GameplayTurnControlRequestDto>(request) {
                Ok(control) => {
                    let response = host.execute_turn_control(control);
                    (200, response)
                }
                Err(diagnostic) => invalid_request(host, diagnostic),
            }
        }
        (&Method::Post, "/api/session/checkpoint/restore") => {
            let response = host.restore_latest_checkpoint();
            (if response.ok { 200 } else { 409 }, response)
        }
        (&Method::Post, "/api/session/replay") => {
            let response = host.replay_archive();
            (if response.ok { 200 } else { 409 }, response)
        }
        _ => {
            let mut response = host.status();
            response.ok = false;
            response.diagnostics = vec![PlayDiagnosticDto {
                stage: "transport".to_owned(),
                severity: "error".to_owned(),
                code: "PLAY_ROUTE_NOT_FOUND".to_owned(),
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
) -> Result<PreparedPlayBundleCompileRequestDto, Box<PlayDiagnosticDto>> {
    decode_request(request)
}

fn decode_request<Value: DeserializeOwned>(
    request: &mut Request,
) -> Result<Value, Box<PlayDiagnosticDto>> {
    let mut body = String::new();
    request
        .as_reader()
        .read_to_string(&mut body)
        .map_err(|error| Box::new(request_diagnostic(error.to_string())))?;
    serde_json::from_str(&body).map_err(|error| Box::new(request_diagnostic(error.to_string())))
}

fn invalid_request(
    host: &PlayHost,
    diagnostic: Box<PlayDiagnosticDto>,
) -> (u16, PlayWorkspaceResponseDto) {
    let mut response = host.status();
    response.ok = false;
    response.diagnostics = vec![*diagnostic];
    (400, response)
}

fn request_diagnostic(message: String) -> PlayDiagnosticDto {
    PlayDiagnosticDto {
        stage: "transport".to_owned(),
        severity: "error".to_owned(),
        code: "PLAY_BUNDLE_COMPILE_REQUEST_INVALID".to_owned(),
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
