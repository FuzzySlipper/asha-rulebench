# Rulebench Fixtures

`rulebench-fixtures` owns Rulebench-only scenario packages, deterministic
receipts, transcripts, scripts, and their regression manifests. It is not
portable rule authority and must never be imported by a game consumer.

Each package declares its expected authority evidence in Rust data. Generated
TypeScript catalog and session files are projections of that evidence, not the
golden source. Check them with:

```bash
pnpm run catalog:check
pnpm run session:check
```

After an intentional, reviewed Rust evidence change, regenerate only through:

```bash
pnpm run catalog:write
pnpm run session:write
```

Then re-run both checks and the full `pnpm run verify` gate before committing.

## Non-Claims

This crate does not define gameplay behavior, accept TypeScript callbacks, or
serve as a generic game content-pack format. Rust rule crates remain the
authority for validation and resolution.
