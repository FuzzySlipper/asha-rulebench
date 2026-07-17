# Viewer Evidence Boundaries

The product viewer reads current Rust authority evidence from the running
process host. It has no generated scenario/session catalog fallback.

## Product readback path

`rulebench-product-content` composes named scenario cases and session samples.
At host startup, registered cases are executed through Rust authority and
exposed through versioned viewer routes. `RulebenchLiveTransport`, the store,
domain projection, and UI consume those routes with cancellation, stale-result
suppression, classified errors, and explicit retry.

An unavailable route is an error state. TypeScript may not substitute copied
scenario/session results or mutate authority state.

## Generated artifacts

`libs/protocol/src/generated/api-types.ts` is the only committed Rust-generated
product artifact in this path. It is a wire contract, not evidence that a host
is available or a scenario succeeded. Update it only through
`pnpm run generated:write` and verify it with `pnpm run generated:check`.

Scenario, session, capability, regression, and certification artifacts are
owned downstream by `FuzzySlipper/asha-rulebench-testing` and are not product
runtime inputs.

## Evolution rules

Add viewer fields to Rust protocol DTOs and regenerate the TypeScript contract.
Keep bridge methods host-neutral and HTTP/process/JSON concerns in the process
host. New product viewer flows must use the live transport and store boundary.
