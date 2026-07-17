# Validation Evidence Template

Use this structure in Den implementation, validation, and review handoffs. The
goal is to make the selected risk tier and its non-claims inspectable without
copying generated inventory counts into task prose.

```text
Validation tier:
- focused | blocking | certification | certification with live artifacts

Risk classification:
- Changed owners/surfaces:
- Focused profiles selected:
- Why no broader profile was required:

Commands and results:
- Focused:
- Blocking `pnpm run verify`:
- Certification `pnpm run certify`:
- Exact-SHA GitHub checks:

Browser evidence:
- Deterministic workflows executed:
- Cache receipt versus actual browser execution:
- Managed `BASE_URL`:
- Live artifact command:
- Screenshots/evidence packet inspected:
- Rendered behavior observed:

Governance evidence:
- Claims/limitations receipt:
- Den document handles reviewed:
- Limitation snapshot provenance:

Proof routing:
- Retained duplicate and its distinct defect class:
- Deleted/narrowed/generated proof:
- Retained detection path:

Non-claims and residual risk:
- What this run did not establish:

Scope accounting:
- Acceptance criteria satisfied:
- Intentionally deferred work:
- Follow-up tasks and classification:
- Is any follow-up required for parent acceptance? no | yes | uncertain
```

When classification is uncertain, run the blocking gate. Use certification for
milestones, releases, governed ASHA changes, compatibility/migration changes,
or proof removal. User-visible milestone/release claims additionally require
the managed live-artifact mode and human inspection of the rendered evidence.
