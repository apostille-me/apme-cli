# apme-cli

flags-2-env operator CLI for Apostille Me health, listing, and WebSocket event watching.

**Product:** Apostille Me — Case operations for visa and apostille consulting.

Track sanitized client references, document workflows, destination jurisdictions, appointments, deadlines, and case events for a visa and apostille consulting firm.

## Safety and production boundary

This software is an operational starter and does not provide legal advice. Keep identity documents and sensitive case files out of logs and this bootstrap data model; production use requires encryption, access controls, retention rules, auditability, and jurisdiction-specific professional review.

This repository is an executable bootstrap, not a production deployment. Before live
use, add authentication, tenant authorization, rate limits, durable migrations,
observability, backups, incident response, dependency review, and secret management.
## Examples

```bash
cargo run -- health
cargo run -- --api-url http://127.0.0.1:8080 list
cargo run -- watch
```

Precedence is `CLI > environment > schema default`. The CLI audits
`.cli-flags.toml`, rejects unknown options and parse errors, and crosses into typed
configuration once before network work.
