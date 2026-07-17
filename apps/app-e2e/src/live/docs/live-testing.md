# Live Testing

Live scenarios are opt-in and gated by `LIVE_RUN=1`. Start the app through `den-serve` so it binds LAN-facing and records a managed session:

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run e2e:live
```

`e2e:live` is a compatibility alias for the artifact-only
`e2e:live-artifacts` group. Deterministic integration journeys belong to
`e2e:certification` and are not rerun merely because they use a live Rust host.
For milestone or release certification, use the canonical composition:

```bash
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run certify -- --require-live
```

Use the printed `local:` URL for Playwright probes. Report the printed `lan:`
URL for human inspection from another machine.

Each scenario must write an evidence packet, milestone screenshots when the claim is visual, console and page error dumps, visible text, and explicit non-claims.

Completion evidence report:

```text
Live scenario:
Command:
Backend/profile:
Artifacts:
Screenshots inspected:
Rendered behavior observed:
Evidence packet:
Timeline notes:
Supporting checks:
Non-claims / residual risk:
```
