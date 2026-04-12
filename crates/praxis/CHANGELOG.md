# Changelog

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
