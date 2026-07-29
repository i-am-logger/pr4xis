# Security Policy

pr4xis is a reasoning-engine library, CLI, and a browser-only WASM demo — there's
no server component and no network attack surface in the traditional sense. The
security-relevant categories here are:

- **Memory/deserialization safety** — `pr4xis-runtime`'s archive/lens code
  (`crates/pr4xis-runtime/src/lens/`) uses `unsafe` for zero-copy `rkyv`
  deserialization of `.prx`/`.prx.gz` ontology archives, including ones fetched
  over the network by the browser demo. A crafted archive that bypasses the
  fail-closed hash-validation load gate would be a real vulnerability.
- **Dependency vulnerabilities** — standard `RUSTSEC` advisories in the
  dependency tree.
- **Incorrect axioms presented as proven** — for a project whose entire pitch is
  "every claim carries a proof path back to its axioms," a bug that lets an
  unproven or unsound claim through as if verified is a trust/integrity issue
  worth reporting privately first, even though it isn't memory-unsafe.

## Reporting a Vulnerability

Please report security issues privately rather than opening a public issue —
[GitHub Security Advisories](https://github.com/i-am-logger/pr4xis/security/advisories/new)
if private vulnerability reporting is enabled on the repo, otherwise contact the
maintainer directly via GitHub ([@i-am-logger](https://github.com/i-am-logger)).

Include what you'd include in any report: the affected version/commit, a
minimal reproduction, and the impact as you see it. There's no bug bounty —
this is a solo-maintained open-source project — but reports are taken
seriously and credited unless you ask otherwise.

## Supported Versions

Only the latest published release is supported. There is no LTS branch.
