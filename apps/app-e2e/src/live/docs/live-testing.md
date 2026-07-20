# Live Testing

Live scenarios are opt-in and gated by `LIVE_RUN=1`. Start the app through `den-serve` so it binds LAN-facing and records a managed session:

```bash
den-serve up asha-rulebench -repo /home/dev/asha-rulebench
BASE_URL=<local-url-from-den-serve> LIVE_RUN=1 pnpm run e2e:live
```

`e2e:live` is a compatibility alias for the `e2e:live-artifacts` group. The
current scenario inspects inactive startup and explicit compilation, authors an
artifact-pinned encounter, then plays several alternating authority-owned turns
in the visible desktop and narrow combat workspace with system-supplied
automatic rolls, an inline reaction, and exact Rust replay. Artifact and replay
evidence remain secondary dialogs. It does not claim persistent setup
libraries, process-restart persistence, multiplayer, AI, migration, or
downstream certification.

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
