use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::{HttpMethod, HttpRequest, HttpResponse, ProcessHostRouter};

const MAX_HEADER_BYTES: usize = 32 * 1024;
const MAX_BODY_BYTES: usize = 1024 * 1024;

#[derive(Debug)]
pub enum ServerError {
    Io(std::io::Error),
    MalformedRequest(String),
}

impl Display for ServerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::Io(error) => write!(formatter, "host I/O error: {error}"),
            ServerError::MalformedRequest(message) => {
                write!(formatter, "malformed HTTP request: {message}")
            }
        }
    }
}

impl std::error::Error for ServerError {}

impl From<std::io::Error> for ServerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn serve_until(
    listener: TcpListener,
    mut router: ProcessHostRouter,
    shutdown: Arc<AtomicBool>,
) -> Result<(), ServerError> {
    listener.set_nonblocking(true)?;
    while !shutdown.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_read_timeout(Some(Duration::from_secs(5)))?;
                stream.set_write_timeout(Some(Duration::from_secs(5)))?;
                serve_connection(&mut stream, &mut router)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(ServerError::Io(error)),
        }
    }
    Ok(())
}

fn serve_connection(
    stream: &mut TcpStream,
    router: &mut ProcessHostRouter,
) -> Result<(), ServerError> {
    let response = match read_request(stream) {
        Ok(request) => router.handle(&request),
        Err(error) => HttpResponse::json(
            400,
            format!(
                "{{\"kind\":\"transport\",\"code\":\"malformedHttpRequest\",\"message\":{},\"retryable\":false}}",
                serde_json::to_string(&error.to_string()).unwrap_or_else(|_| "\"Malformed request.\"".to_string())
            )
            .into_bytes(),
        ),
    };
    write_response(stream, &response)
}

fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, ServerError> {
    let mut bytes = Vec::new();
    let header_end = loop {
        if bytes.len() > MAX_HEADER_BYTES {
            return Err(ServerError::MalformedRequest(
                "header exceeds the configured limit".to_string(),
            ));
        }
        let mut buffer = [0_u8; 4096];
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            return Err(ServerError::MalformedRequest(
                "connection closed before headers completed".to_string(),
            ));
        }
        bytes.extend_from_slice(&buffer[..count]);
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
    };

    let header_text = String::from_utf8(bytes[..header_end].to_vec())
        .map_err(|_| ServerError::MalformedRequest("headers must be valid UTF-8".to_string()))?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| ServerError::MalformedRequest("missing request line".to_string()))?;
    let mut request_parts = request_line.split_whitespace();
    let method = match request_parts.next() {
        Some("GET") => HttpMethod::Get,
        Some("POST") => HttpMethod::Post,
        Some("DELETE") => HttpMethod::Delete,
        Some(method) => {
            return Err(ServerError::MalformedRequest(format!(
                "unsupported method {method}"
            )))
        }
        None => return Err(ServerError::MalformedRequest("missing method".to_string())),
    };
    let path = request_parts
        .next()
        .ok_or_else(|| ServerError::MalformedRequest("missing request path".to_string()))?
        .to_string();
    if request_parts.next() != Some("HTTP/1.1") {
        return Err(ServerError::MalformedRequest(
            "only HTTP/1.1 is supported".to_string(),
        ));
    }

    let mut headers = BTreeMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let Some((name, value)) = line.split_once(':') else {
            return Err(ServerError::MalformedRequest(
                "header is missing a colon".to_string(),
            ));
        };
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
    }
    let content_length = headers
        .get("content-length")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| ServerError::MalformedRequest("invalid content-length".to_string()))
        })
        .transpose()?
        .unwrap_or_default();
    if content_length > MAX_BODY_BYTES {
        return Err(ServerError::MalformedRequest(
            "body exceeds the configured limit".to_string(),
        ));
    }

    while bytes.len() - header_end < content_length {
        let mut buffer = [0_u8; 4096];
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            return Err(ServerError::MalformedRequest(
                "connection closed before body completed".to_string(),
            ));
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    let body = bytes[header_end..header_end + content_length].to_vec();

    Ok(HttpRequest {
        method,
        path,
        headers,
        body,
    })
}

fn write_response(stream: &mut TcpStream, response: &HttpResponse) -> Result<(), ServerError> {
    let status_text = match response.status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        422 => "Unprocessable Content",
        _ => "Internal Server Error",
    };
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n",
        response.status,
        status_text,
        response.content_type,
        response.body.len()
    )?;
    stream.write_all(&response.body)?;
    stream.flush()?;
    Ok(())
}
