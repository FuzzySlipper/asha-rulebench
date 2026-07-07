# UI Pattern Bootstrap Template

Bare-file Angular/Nx bootstrap for projects following `den:patch/rusty-view-ui-architecture-pattern`.

Start a new project by copying this directory without git history, then run:

```bash
pnpm install
pnpm run init -- <project-name>
pnpm run verify
```

For local development:

```bash
pnpm run dev
```

For opt-in live evidence:

```bash
pnpm run serve:local
BASE_URL=<printed-url> LIVE_RUN=1 pnpm run e2e:live
```

The template ships only the reusable skeleton: one wired layer lib, generated boundary rules, pattern conformance checks, docs command checks, deterministic smoke E2E, and a generic live artifact harness.
