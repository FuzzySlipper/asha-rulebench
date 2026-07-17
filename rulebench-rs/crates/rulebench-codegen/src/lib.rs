//! Rulebench product protocol generation support.
//!
//! Exhaustive scenario/session artifact rendering moved to
//! `asha-rulebench-testing`.

#![forbid(unsafe_code)]

pub fn render_protocol_types() -> String {
    rulebench_protocol::render_api_types()
}

#[cfg(test)]
mod tests {
    use super::render_protocol_types;
    const GENERATED_PROTOCOL_TYPES: &str =
        include_str!("../../../../libs/protocol/src/generated/api-types.ts");

    #[test]
    fn protocol_renderer_matches_the_committed_artifact() {
        assert_eq!(render_protocol_types(), GENERATED_PROTOCOL_TYPES);
    }
}
