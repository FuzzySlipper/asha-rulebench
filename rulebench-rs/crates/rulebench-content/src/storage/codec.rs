use std::collections::BTreeMap;

use super::ContentStorageRecord;
use crate::{
    ContentDefinitionKind, ContentDefinitionReference, ContentFingerprint, ContentPackProvenance,
    ContentPackReference, ContentPackSourceKind,
};

const STORAGE_FORMAT: &str = "rulebench-content-storage.v0";
const PAYLOAD_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-content-payload.v0";

pub(super) fn record_file_stem(reference: &ContentPackReference) -> String {
    format!(
        "{}-{}-{}",
        hex_encode(reference.id.as_bytes()),
        hex_encode(reference.version.as_bytes()),
        reference.fingerprint.value
    )
}

pub(super) fn fingerprint_payload(payload: &[u8]) -> ContentFingerprint {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in PAYLOAD_FINGERPRINT_ALGORITHM
        .as_bytes()
        .iter()
        .chain(payload.iter())
    {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    ContentFingerprint {
        algorithm: PAYLOAD_FINGERPRINT_ALGORITHM.to_string(),
        value: format!("{hash:016x}"),
    }
}

pub(super) fn encode_record(record: &ContentStorageRecord) -> Vec<u8> {
    let mut lines = vec![
        pair("format", STORAGE_FORMAT),
        encoded_pair("id", &record.reference.id),
        encoded_pair("version", &record.reference.version),
        encoded_pair(
            "fingerprintAlgorithm",
            &record.reference.fingerprint.algorithm,
        ),
        encoded_pair("fingerprintValue", &record.reference.fingerprint.value),
        encoded_pair("title", &record.title),
        encoded_pair("summary", &record.summary),
        pair("sourceKind", record.provenance.source_kind.code()),
        encoded_pair("sourceId", &record.provenance.source_id),
        encoded_pair(
            "authoredBy",
            record.provenance.authored_by.as_deref().unwrap_or(""),
        ),
        encoded_pair("rulesetId", &record.ruleset_id),
        encoded_pair("rulesetVersion", &record.ruleset_version),
        encoded_pair(
            "payloadFingerprintAlgorithm",
            &record.payload_fingerprint.algorithm,
        ),
        encoded_pair("payloadFingerprintValue", &record.payload_fingerprint.value),
    ];
    for definition in &record.definitions {
        lines.push(format!(
            "definition={}:{}",
            definition.kind.code(),
            hex_encode(definition.id.as_bytes())
        ));
    }
    lines.push(String::new());
    lines.join("\n").into_bytes()
}

pub(super) fn decode_record(bytes: &[u8]) -> Result<ContentStorageRecord, String> {
    let source = std::str::from_utf8(bytes).map_err(|_| "record is not UTF-8".to_string())?;
    let mut fields = BTreeMap::new();
    let mut definitions = Vec::new();
    for line in source.lines() {
        let Some((key, value)) = line.split_once('=') else {
            return Err("record line has no field separator".to_string());
        };
        if key == "definition" {
            definitions.push(decode_definition(value)?);
        } else if fields.insert(key, value).is_some() {
            return Err(format!("duplicate record field {key}"));
        }
    }
    if required(&fields, "format")? != STORAGE_FORMAT {
        return Err("unsupported record format".to_string());
    }

    definitions.sort();
    if definitions.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("duplicate definition index entry".to_string());
    }
    let source_kind = match required(&fields, "sourceKind")? {
        "embedded" => ContentPackSourceKind::Embedded,
        "authoredFile" => ContentPackSourceKind::AuthoredFile,
        "bridgeSubmission" => ContentPackSourceKind::BridgeSubmission,
        _ => return Err("unknown content source kind".to_string()),
    };
    let authored_by = decode_field(&fields, "authoredBy")?;

    Ok(ContentStorageRecord {
        reference: ContentPackReference {
            id: decode_field(&fields, "id")?,
            version: decode_field(&fields, "version")?,
            fingerprint: ContentFingerprint {
                algorithm: decode_field(&fields, "fingerprintAlgorithm")?,
                value: decode_field(&fields, "fingerprintValue")?,
            },
        },
        title: decode_field(&fields, "title")?,
        summary: decode_field(&fields, "summary")?,
        provenance: ContentPackProvenance {
            source_kind,
            source_id: decode_field(&fields, "sourceId")?,
            authored_by: (!authored_by.is_empty()).then_some(authored_by),
        },
        ruleset_id: decode_field(&fields, "rulesetId")?,
        ruleset_version: decode_field(&fields, "rulesetVersion")?,
        definitions,
        payload_fingerprint: ContentFingerprint {
            algorithm: decode_field(&fields, "payloadFingerprintAlgorithm")?,
            value: decode_field(&fields, "payloadFingerprintValue")?,
        },
    })
}

fn decode_definition(value: &str) -> Result<ContentDefinitionReference, String> {
    let (kind, encoded_id) = value
        .split_once(':')
        .ok_or_else(|| "definition entry has no kind separator".to_string())?;
    Ok(ContentDefinitionReference {
        kind: ContentDefinitionKind::from_code(kind)
            .ok_or_else(|| format!("unknown definition kind {kind}"))?,
        id: decode_hex_string(encoded_id)?,
    })
}

fn required<'a>(fields: &BTreeMap<&'a str, &'a str>, key: &str) -> Result<&'a str, String> {
    fields
        .get(key)
        .copied()
        .ok_or_else(|| format!("missing record field {key}"))
}

fn decode_field(fields: &BTreeMap<&str, &str>, key: &str) -> Result<String, String> {
    decode_hex_string(required(fields, key)?)
}

fn pair(key: &str, value: &str) -> String {
    format!("{key}={value}")
}

fn encoded_pair(key: &str, value: &str) -> String {
    pair(key, &hex_encode(value.as_bytes()))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn decode_hex_string(value: &str) -> Result<String, String> {
    if value.len() % 2 != 0 {
        return Err("hex field has odd length".to_string());
    }
    let bytes = value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let encoded = std::str::from_utf8(pair).map_err(|_| "invalid hex field".to_string())?;
            u8::from_str_radix(encoded, 16).map_err(|_| "invalid hex field".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    String::from_utf8(bytes).map_err(|_| "hex field is not UTF-8".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip_preserves_arbitrary_utf8_and_separators() {
        let value = "content=id\nversion=one";
        assert_eq!(
            decode_hex_string(&hex_encode(value.as_bytes())),
            Ok(value.to_string())
        );
    }
}
