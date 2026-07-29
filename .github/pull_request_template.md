## What does this change and why?

<!-- The "why" matters more than the "what" — the diff already shows what changed. -->

## Checklist

- [ ] `dev-ci` passes locally (fmt, clippy, docs, tests, constitution gate — see [CONTRIBUTING.md](../CONTRIBUTING.md))
- [ ] No `todo!()` / `unimplemented!()` / `debug_assert!()` introduced
- [ ] New domain knowledge goes through an ontology, not inline string matching or magic numbers
- [ ] If this touches `pr4xis-domains` tests: each new test declares its guarantee via `#[pr4xis::praxis_value(..)]`
- [ ] I agree this contribution is licensed under [CC BY-NC-SA 4.0](../LICENSE)
