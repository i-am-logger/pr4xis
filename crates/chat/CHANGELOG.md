# Changelog

## [0.5.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-chat-v0.4.2...pr4xis-chat-v0.5.0) (2026-04-16)


### Features

* define_ontology! being: clause + register all 108 ontologies ([#76](https://github.com/i-am-logger/pr4xis/issues/76)) ([#84](https://github.com/i-am-logger/pr4xis/issues/84)) ([1b27fc9](https://github.com/i-am-logger/pr4xis/commit/1b27fc974e1a4b018542ad4ea6ae57e3f4d9f561))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.7.0 to 0.8.0
    * pr4xis-domains bumped from 0.10.0 to 0.11.0

## [0.4.2](https://github.com/i-am-logger/pr4xis/compare/pr4xis-chat-v0.4.1...pr4xis-chat-v0.4.2) (2026-04-16)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis-domains bumped from 0.9.0 to 0.10.0

## [0.4.1](https://github.com/i-am-logger/pr4xis/compare/pr4xis-chat-v0.4.0...pr4xis-chat-v0.4.1) (2026-04-15)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis-domains bumped from 0.8.0 to 0.9.0

## [0.4.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-chat-v0.3.0...pr4xis-chat-v0.4.0) (2026-04-13)


### Features

* enforce ontology patterns — define_ontology! everywhere, 4851 tests ([63031ee](https://github.com/i-am-logger/pr4xis/commit/63031ee4bc8b96f874b9e3b0e192e881494265f0))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.6.0 to 0.7.0
    * pr4xis-domains bumped from 0.7.0 to 0.8.0

## [0.3.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-chat-v0.2.0...pr4xis-chat-v0.3.0) (2026-04-12)


### Features

* restructure to academic hierarchy (DOLCE-aligned) ([44997fa](https://github.com/i-am-logger/pr4xis/commit/44997fae2ed61f693b592839cc8f27efb4cc35bc))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * pr4xis bumped from 0.5.0 to 0.6.0
    * pr4xis-domains bumped from 0.6.0 to 0.7.0

## [0.2.0](https://github.com/i-am-logger/pr4xis/compare/pr4xis-chat-v0.1.5...pr4xis-chat-v0.2.0) (2026-04-12)


### Features

* debug trace shows as collapsible pipeline tree in chat UI ([ffd1b2a](https://github.com/i-am-logger/pr4xis/commit/ffd1b2aff7f594c08c2bc9f9a0ce0ab5dfff356a))
* distinguish ontology vs fallback responses visually ([f3b40d5](https://github.com/i-am-logger/pr4xis/commit/f3b40d5bc0adbca3d7ea2513d0948c5fbe63974f))
* metacognition explores taxonomy when concepts found but parse fails ([aa685ac](https://github.com/i-am-logger/pr4xis/commit/aa685accb1bb8a98856aded4ef7ea4e01a49b69a))
* praxis-chat + praxis-wasm crates, logo in README ([d492e89](https://github.com/i-am-logger/pr4xis/commit/d492e89f10341fe5f34d23493269368f0aa56d77))
* proper ontology trace — each step reports what it did with status ([f7b1b7f](https://github.com/i-am-logger/pr4xis/commit/f7b1b7f7bfb5502e6c851236893e9d0a0d4a4241))
* proper ontology trace — each step reports what it did with status ([1605d7e](https://github.com/i-am-logger/pr4xis/commit/1605d7e94d7c1af2debce45ec7534257d9c67bd2))
* proper ontology trace — each step reports what it did with status ([20739ae](https://github.com/i-am-logger/pr4xis/commit/20739ae4f4b9e3926e6b22747add976f3e01249c))
* pure response functions — no &mut Trace threading, all return ResponseResult ([b953f22](https://github.com/i-am-logger/pr4xis/commit/b953f22c99ad0911fde0e87d82d7f9880749a07b))
* readable pipeline trace with labeled sections ([aeb7c70](https://github.com/i-am-logger/pr4xis/commit/aeb7c70eb7887d447634fc8ceaed10e21d7354c8))
* rename praxis → pr4xis across entire codebase ([5e971f7](https://github.com/i-am-logger/pr4xis/commit/5e971f77ac3041a5e35209216d09f41e55cf8a0d))
* rich taxonomy responses with path, definitions, and subtypes ([4bd0fc8](https://github.com/i-am-logger/pr4xis/commit/4bd0fc84bdf99a03ea7d7423cc1ad5f073f6d3cc))
* self-model ontology, CYK chart parser, adjunction, response generation ([0f67d8d](https://github.com/i-am-logger/pr4xis/commit/0f67d8d629065b3e76d3acd4567a03e9cc346c7e))
* trace shows Lambek notation (S[q]/NP) not Rust debug format ([3f24ff5](https://github.com/i-am-logger/pr4xis/commit/3f24ff5493ba958594ed72f9d4717936acfcb957))


### Bug Fixes

* chat now answers questions correctly, add metacognition trace ([84c5991](https://github.com/i-am-logger/pr4xis/commit/84c59910a2a64a3f77ffe9477d7d77b25d561778))
* remove hardcoded debug test strings from chat ([01d2f97](https://github.com/i-am-logger/pr4xis/commit/01d2f97840a1064edf11a01f135c9cb4b491ba5d))
* trace shows Lambek notation (S[q]/NP) not Rust debug format ([d369326](https://github.com/i-am-logger/pr4xis/commit/d369326b7be16ec55bb6b23083603791107ae22e))
* update release-please config for pr4xis rename + add version to path deps ([ff60744](https://github.com/i-am-logger/pr4xis/commit/ff60744ee9dbdd64d2a964b39c286253216e9a58))
* WASM panic — std::time::Instant unsupported on wasm32 ([e711cdc](https://github.com/i-am-logger/pr4xis/commit/e711cdc16ac8b5ed1c235ef7983810448641809b))
