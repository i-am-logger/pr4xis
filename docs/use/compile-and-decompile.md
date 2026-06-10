# Compile & Decompile

This page is the operator's guide for the two legs of the praxis compiler: `pr4xis compile`, which turns every [registered source](register-a-source.md) into a verifiable [`.prx`](../reference/glossary.md#prx) archive, and `pr4xis decompile`, which turns an archive back into the exact original source bytes. The **completeness meter** (`pr4xis decompile --meter`) is the honesty report over the whole set: per source, what round-trip fidelity is declared and what is actually achieved.

Both commands operate on the registry (`praxis.toml`) and the lock (`praxis.lock`); `compile` consumes the source files that `pr4xis update` provisions, so run that first (or pass `--update`).

## What `compile` produces

```bash
pr4xis compile
```

emits one content-addressed `.prx.gz` per registered OWL vocabulary, U.S. Code title, and WordNet language into the build cache at `<workspace>/.prx-cache/`. Two artifact families come out, serving two different consumers:

- **The compact runtime caches** — `.prx-cache/usc-compact/` and `.prx-cache/wordnet-compact/`. These are the parse-once fast-load archives the runtime loaders read: `UsCode::loaded()` and `english_loaded()` load a pinned compact archive in milliseconds instead of re-parsing the source XML in every process. The compact codec is dependency-free bit-packing, so its content addresses are portable — stable across toolchains and targets.
- **The distribution envelopes** — `.prx-cache/ontologies/` (OWL), `.prx-cache/usc/`, and `.prx-cache/wordnet/`. These rkyv envelopes carry the typed ontology graph plus a content-addressed concrete-syntax complement; they are what `pr4xis decompile` reads to regenerate the source. Their content address (a `MerkleRoot` over the rkyv bytes) is a build output, so it is pinned per toolchain rather than portably.

Each line of output names the artifact, its size, and its content address:

```text
  compiled  cito@2.8.1  …  bytes  2fa4c96c12ea…
  compiled  usc_title_18@pl-119-90  …  bytes  ed47add31553…
  …
26 archive(s) (10 compact), … bytes total → /…/.prx-cache
verified 26 archive(s) against praxis.lock pins (0 unpinned, no fast path).
```

A registered, pinned source that is **not on disk** is an error, not a silent skip — the "forgot to run `pr4xis update`" failure is reported by name. `pr4xis compile --update` provisions the missing sources first instead of erroring; CI provisions separately.

### `--compact` — the fast CI mode

```bash
pr4xis compile --compact
```

emits (and verifies) only the compact runtime caches, skipping the heavy, toolchain-coupled envelopes. This is the CI check: it re-derives and verifies the committed `[compact_archive_signatures]` pins for **all** U.S. Code titles — including the giants the unit-test budget caps out of — in seconds.

## The pin / verify discipline

`praxis.lock` carries two archive-pin sections alongside the source `[hashes]`:

- **`[archive_signatures]`** — the `MerkleRoot` of each rkyv envelope.
- **`[compact_archive_signatures]`** — the portable content address of each compact archive. The runtime's [fail-closed load gate](../reference/glossary.md#fail-closed-load-gate) checks the installed compact bytes against this pin; a title (or the English archive) takes the fast path only when it is pinned here.

The discipline mirrors `pr4xis update`'s `[hashes]` handling:

- **Default = verify (CI-safe, writes nothing).** Every emitted archive's content address must equal its committed pin. Any drift **fails closed** with the offending sources named:

  ```text
  pr4xis compile: praxis.lock pin drift (1 archive(s)) — re-run `pr4xis compile --lock`
  after confirming the change is intended:
    cito@2.8.1 [archive_signatures]: emitted 9f3a… ≠ pinned 2fa4…
  ```

  An *unpinned* archive is reported but never fails — it simply gets no fast path until it is pinned.

- **`--lock` = the deliberate re-pin (maintainer write mode).** Writes each emitted archive's content address into the corresponding lock section, preserving comments and key ordering. Run it locally after a source or codec change you have confirmed is intended — never in CI:

  ```bash
  pr4xis compile --lock
  ```

So a source file, codec, or envelope-layout change that silently alters any archive is caught by the next plain `pr4xis compile`; the only way the pins move is a human running `--lock` on purpose.

## What `decompile` gives you

```bash
pr4xis decompile cito                       # → cito-2.8.1.owl
pr4xis decompile usc_title_18 --out t18.xml # → t18.xml
pr4xis decompile english_wordnet            # → english-wordnet XML
```

`decompile` is the inverse leg: it resolves the registered source by name (`pr4xis update --list` shows the names), loads the envelope `compile` wrote into `.prx-cache/`, regenerates the **original source bytes**, writes them to `--out` (or `<name>-<version>.<ext>` in the current directory), and prints the achieved round-trip fidelity:

```text
decompiled cito@2.8.1 → cito-2.8.1.owl (… bytes)
  round-trip fidelity: ByteExactGraphFaithful (regenerated from the ontology graph alone)
```

Routing is registry-derived, not byte-sniffing: the source's content type selects the reconstruct leaf (OWL RDF/XML, USLM XML, or WN-LMF XML) inside one uniform decompile op. Every reconstruction passes a content-address honesty gate — the regenerated bytes must re-derive the recorded source address, or the load is refused.

The law this realises, proven per source by the test suite over the real bytes:

```text
hash(decompile(compile(source))) == hash(source)
```

Today **17 registered sources** carry a `.prx` compile/decompile pair — 6 OWL vocabularies, 9 U.S. Code titles, and 2 LMF lexicons (the English WordNet and the bundled US legal lexicon) — and each round-trips **byte-for-byte**.

There are two fidelity tiers, both byte-exact:

- **`RawBytesComplementFloor`** — the bytes come back from a stored, content-addressed copy of the source (a *constant complement*) inside the archive. Real, cryptographically witnessed exactness, but from a stored side-channel.
- **`ByteExactGraphFaithful`** — the bytes are regenerated from the typed ontology graph plus a small concrete-syntax complement, with **no stored raw blob**. This is the tier the per-source lens registrations declare for all 17 sources today.

## The completeness meter

```bash
pr4xis decompile --meter
```

prints the honesty report: one line per registered `.prx` source, stating the tier its round-trip reaches and — for any source still on the floor — the named writer gap that remains:

```text
biro@1.1.1: graph-faithful
cito@2.8.1: graph-faithful
…
usc_title_42@pl-119-90: graph-faithful (declared) — byte-exact proof in the slow / all-sources lane
…
decompile completeness: … graph-faithful, … still on the stored-complement floor (…)
```

Two properties make it a report you can trust:

- **It cannot over-claim.** Each row carries both the *declared* tier (what the source's registered lens promises) and the *achieved* tier (what the emitted archive actually carries, as measured by the round-trip harness). A test asserts they agree for every provisioned source, so an archive claiming graph-faithfulness it does not achieve is a test failure, not a meter line.
- **It does not guess.** A source whose corpus is not provisioned on this machine, or whose size defers it to the slower all-sources test lane, is stated as such rather than credited with a tier the fast harness did not measure.

The meter is non-failing — it never blocks CI. It exists so the remaining distance to a fully graph-only compiler is always stated per source, never averaged away.

## Related

- [Register a Source](register-a-source.md) — the manifest/lock/`pr4xis update` workflow that provisions what `compile` consumes
- [Test Architecture](../understand/test-architecture.md) — how the compact archives serve as the parse-once fast path for the test suite
- [Glossary](../reference/glossary.md) — [`.prx`](../reference/glossary.md#prx), [Archive](../reference/glossary.md#archive), [IntegrityClaim](../reference/glossary.md#integrityclaim), [Fail-closed load gate](../reference/glossary.md#fail-closed-load-gate)

---

- **Document date:** 2026-06-09
