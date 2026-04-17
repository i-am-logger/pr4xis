# Changelog

## [0.10.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-v0.9.0...pr4xis-v0.10.0) (2026-04-17)


### Features

* compose API — runtime ontology composition via Korporator ([#103](https://github.com/i-am-logger/pr4xis/issues/103)) ([#108](https://github.com/i-am-logger/pr4xis/issues/108)) ([848d986](https://github.com/i-am-logger/pr4xis/commit/848d986457f82a758f3315c049063b53962ed00f))
* ontology! proc macro — fluent syntax with static codegen ([#103](https://github.com/i-am-logger/pr4xis/issues/103)) ([#109](https://github.com/i-am-logger/pr4xis/issues/109)) ([b66edcc](https://github.com/i-am-logger/pr4xis/commit/b66edccb3725a5f417bd63febf240ee1a81017a0))
* typed Vocabulary — OntologyName, ModulePath, structured Citation ([#111](https://github.com/i-am-logger/pr4xis/issues/111)) ([a05b34c](https://github.com/i-am-logger/pr4xis/commit/a05b34c061a8c0c784a72c20a8afe1deec7fae7b))

## [0.9.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-v0.8.0...pr4xis-v0.9.0) (2026-04-16)


### Features

* Ontolex-Lemon, consciousness C1×C2, complete functor chain, Vocabulary API ([#88](https://github.com/i-am-logger/pr4xis/issues/88)) ([#104](https://github.com/i-am-logger/pr4xis/issues/104)) ([d3a5a46](https://github.com/i-am-logger/pr4xis/commit/d3a5a46aca23292c85078390499b696c9bff3c0e))

## [0.8.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-v0.7.0...pr4xis-v0.8.0) (2026-04-16)


### Features

* define_ontology! being: clause + register all 108 ontologies ([#76](https://github.com/i-am-logger/pr4xis/issues/76)) ([#84](https://github.com/i-am-logger/pr4xis/issues/84)) ([1b27fc9](https://github.com/i-am-logger/pr4xis/commit/1b27fc974e1a4b018542ad4ea6ae57e3f4d9f561))

## [0.7.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-v0.6.0...pr4xis-v0.7.0) (2026-04-13)


### Features

* enforce ontology patterns — define_ontology! everywhere, 4851 tests ([63031ee](https://github.com/i-am-logger/pr4xis/commit/63031ee4bc8b96f874b9e3b0e192e881494265f0))

## [0.6.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-v0.5.0...pr4xis-v0.6.0) (2026-04-12)


### Features

* complete algebraic structure library — 7 new structures ([281f9e3](https://github.com/i-am-logger/pr4xis/commit/281f9e3087a97a988c33bd9deff27956a9cce759))
* define_ontology! clean API — concepts/is_a/has_a/causes/opposes + auto structural axioms ([366f284](https://github.com/i-am-logger/pr4xis/commit/366f28459f606ef56323910decd72b9be085e624))
* define_ontology! macro — generates Category + Taxonomy + Mereology + Causation + Opposition + OntologyMeta ([07a1b54](https://github.com/i-am-logger/pr4xis/commit/07a1b549ecac1d321d679bddc6dfcaee9cb14138))
* derive macros — #[derive(Entity)] + define_category! + define_dense_category! ([e598947](https://github.com/i-am-logger/pr4xis/commit/e598947d89ff36d1e4d84ac09ad1720915034483))
* F-algebra, MonoidalCategory, Optics, MonadTransformer (4 structures) ([22211e2](https://github.com/i-am-logger/pr4xis/commit/22211e2c6d9a71e29d126e066622a74d58d39948))
* integrate algebraic structures into reasoning + tracing ([5f9651d](https://github.com/i-am-logger/pr4xis/commit/5f9651d18238565f7b4f5915327da23e1dfde594))
* integrate Kleisli + anamorphism + Yoneda into causation reasoning ([393d94c](https://github.com/i-am-logger/pr4xis/commit/393d94cf10050d43ec7cd663833596c0bc3ce4b3))
* migrate Ontology impls to structural + domain axiom split ([8a79323](https://github.com/i-am-logger/pr4xis/commit/8a793238c6d70160acc9cedf4e1341de9836386e))
* Monoid + Writer monad + TracedCategory refactor (Moggi 1991, Mac Lane 1971) ([32b6292](https://github.com/i-am-logger/pr4xis/commit/32b6292239c77d0306b15aa2ad16271b1a48e60b))
* Ontology trait — structural + domain axioms merged via monoid ([32ac02b](https://github.com/i-am-logger/pr4xis/commit/32ac02b9ff557fb01d7c9bcc04c227fd07476992))
* Reader + State monads with property-based tests ([3d1b01e](https://github.com/i-am-logger/pr4xis/commit/3d1b01ef72d751e31a2829d9eac120d87c2ccdef))


### Bug Fixes

* clippy clean — no dead code, no unused imports, no stubs ([def3e3e](https://github.com/i-am-logger/pr4xis/commit/def3e3ef7ea816f16826184f6fa77c833d938df9))
* qualify kind refs in define_category! macro (avoid Identity ambiguity) + LOC badge ([4ab4d34](https://github.com/i-am-logger/pr4xis/commit/4ab4d34b2aab9a77d1082288a8bf616940e55e39))

## [0.5.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-v0.4.0...pr4xis-v0.5.0) (2026-04-12)


### Features

* rename praxis → pr4xis across entire codebase ([5e971f7](https://github.com/i-am-logger/pr4xis/commit/5e971f77ac3041a5e35209216d09f41e55cf8a0d))

## [0.4.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-v0.3.0...pr4xis-v0.4.0) (2026-04-12)


### Features

* rename praxis → pr4xis across entire codebase ([5e971f7](https://github.com/i-am-logger/pr4xis/commit/5e971f77ac3041a5e35209216d09f41e55cf8a0d))

## [0.3.0](https://github.com/i-am-logger/pr4xis/compare/praxis-v0.2.0...praxis-v0.3.0) (2026-04-12)


### Features

* build.rs embeds English into binary via codegen ([a15f414](https://github.com/i-am-logger/pr4xis/commit/a15f41467789efc285ef7ba10a46ed6c2ae4a4b1))
* Diagnostics ontology + TracedCategory (writer monad on categories) ([b3beba6](https://github.com/i-am-logger/pr4xis/commit/b3beba6b9e28c2426850fa88fea98a3133b8edea))
* function words as LMF data, extend LmfPos with closed-class types ([0cbffab](https://github.com/i-am-logger/pr4xis/commit/0cbffab9585a3e3f7c202a498e28d3f733c66db9))
* merge sensor-fusion ontologies, add schema/storage/consistency ontologies, praxis-web ([a2fe629](https://github.com/i-am-logger/pr4xis/commit/a2fe629aedbe2363a33b76772e997d1558e14259))
* self-model ontology, CYK chart parser, adjunction, response generation ([0f67d8d](https://github.com/i-am-logger/pr4xis/commit/0f67d8d629065b3e76d3acd4567a03e9cc346c7e))


### Bug Fixes

* codegen was discarding all WordNet definitions (critical bug) ([69ef7d8](https://github.com/i-am-logger/pr4xis/commit/69ef7d819d62852ba3937efd9b76a092ef96ba3e))

## [0.2.0](https://github.com/i-am-logger/praxis/compare/praxis-v0.1.0...praxis-v0.2.0) (2026-04-09)


### Features

* add inference to logic, fix layer violations, update architecture ([0b53d58](https://github.com/i-am-logger/praxis/commit/0b53d5886cdeb48a28f20c1476e164cb77d59cf3))
* DOLCE upper ontology, domain restructure, linguistics, systems thinking, codegen ([368c583](https://github.com/i-am-logger/praxis/commit/368c5835965915495ea43d1b5e3dcbc76b1a93e6))
* Language trait, orthography, morphology, cached reasoning queries ([89daa5a](https://github.com/i-am-logger/praxis/commit/89daa5a45dcdf211e336dceb99503c9a9babda11))


### Bug Fixes

* correct inference semantics and add missing logic re-exports ([b1b6f2e](https://github.com/i-am-logger/praxis/commit/b1b6f2e5dd04ac80b0b592c8e7e667be3658150b))
* resolve all clippy warnings for strict CI ([79ff81b](https://github.com/i-am-logger/praxis/commit/79ff81ba0983283e738516dc8e0be55773add52d))
