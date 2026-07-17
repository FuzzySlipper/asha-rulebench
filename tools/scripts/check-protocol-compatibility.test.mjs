import assert from "node:assert/strict";
import test from "node:test";
import {
  compareProtocolContracts,
  readRustProtocolContract,
  readTypeScriptProtocolContract,
} from "./check-protocol-compatibility.mjs";

test("protocol compatibility accepts one exact cross-owner contract", () => {
  const rust = readRustProtocolContract(
    'pub const PROTOCOL_ID: &str = "asha-rulebench.protocol";\npub const PROTOCOL_VERSION: u32 = 9;',
  );
  const typescript = readTypeScriptProtocolContract(
    'export const RULEBENCH_PROTOCOL_ID = "asha-rulebench.protocol";\nexport const RULEBENCH_PROTOCOL_VERSION = 9;',
  );

  assert.deepEqual(compareProtocolContracts({ rust, typescript }), []);
});

test("protocol compatibility rejects a one-sided version change", () => {
  const failures = compareProtocolContracts({
    rust: { id: "asha-rulebench.protocol", version: 9 },
    typescript: { id: "asha-rulebench.protocol", version: 10 },
    generated: { id: "asha-rulebench.protocol", version: 9 },
  });

  assert.deepEqual(failures, [
    "Protocol version mismatch: rust=9, typescript=10, generated=9.",
  ]);
});

test("protocol compatibility rejects a one-sided identity change", () => {
  const failures = compareProtocolContracts({
    rust: { id: "asha-rulebench.protocol", version: 9 },
    typescript: { id: "asha-rulebench.protocol", version: 9 },
    generated: { id: "asha-rulebench.other", version: 9 },
  });

  assert.deepEqual(failures, [
    "Protocol id mismatch: rust=asha-rulebench.protocol, typescript=asha-rulebench.protocol, generated=asha-rulebench.other.",
  ]);
});
