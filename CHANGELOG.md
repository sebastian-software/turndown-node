# Changelog

## [0.0.12](https://github.com/sebastian-software/turndown-node/compare/turndown-node-v0.0.11...turndown-node-v0.0.12) (2026-01-28)


### Bug Fixes

* achieve output parity with turndown JS for benchmarks ([51ce73c](https://github.com/sebastian-software/turndown-node/commit/51ce73ce12dd65392c1daf417a2ca59ddfbffd61))

## [0.0.11](https://github.com/sebastian-software/turndown-node/compare/turndown-node-v0.0.10...turndown-node-v0.0.11) (2026-01-28)


### Bug Fixes

* handle inline elements at root level in HTML parser ([024f5ec](https://github.com/sebastian-software/turndown-node/commit/024f5ec14b3f41c2926f1133cd96f8b47b651260))
* use workspace dependency for turndown-core ([02594fc](https://github.com/sebastian-software/turndown-node/commit/02594fc46d08582cc7f35a0b35f1efb2986d8d15))

## [0.0.10](https://github.com/sebastian-software/turndown-node/compare/turndown-node-v0.0.9...turndown-node-v0.0.10) (2026-01-28)


### Bug Fixes

* add turndown-core to crates.io publish workflow ([25ec9f6](https://github.com/sebastian-software/turndown-node/commit/25ec9f68651432ac7be7e1ec9583844a40800680))

## [0.0.9](https://github.com/sebastian-software/turndown-node/compare/turndown-node-v0.0.8...turndown-node-v0.0.9) (2026-01-28)


### Features

* add benchmark suite comparing to turndown JS ([f74de35](https://github.com/sebastian-software/turndown-node/commit/f74de351a834ca78e6f51063100380f4593888bc))
* add turndown-core crate with Markdown AST ([8e6e517](https://github.com/sebastian-software/turndown-node/commit/8e6e5175c281467296360ec3a171675e97341a01))
* implement lol_html streaming parser ([35f2740](https://github.com/sebastian-software/turndown-node/commit/35f2740e60137464c767d8929667d842906bc153))

## [0.0.8](https://github.com/sebastian-software/turndown-node/compare/turndown-node-v0.0.7...turndown-node-v0.0.8) (2026-01-27)


### Bug Fixes

* use Node.js 24 (npm v11+) for OIDC publishing ([2d19f13](https://github.com/sebastian-software/turndown-node/commit/2d19f13c122b1a87181983eacdc09012b36bf22e))
* use NPM_TOKEN for npm publishing ([3425f1b](https://github.com/sebastian-software/turndown-node/commit/3425f1b41473320dd852b68fc131184a44234765))

## [0.0.7](https://github.com/sebastian-software/turndown-node/compare/turndown-node-v0.0.6...turndown-node-v0.0.7) (2026-01-27)


### Bug Fixes

* remove registry-url for npm OIDC publishing ([5934d5c](https://github.com/sebastian-software/turndown-node/commit/5934d5c532b85ca063da1480486fbe939b7aad1f))

## [0.0.6](https://github.com/sebastian-software/turndown-node/compare/turndown-node-v0.0.5...turndown-node-v0.0.6) (2026-01-27)


### Bug Fixes

* configure Release Please to update Cargo.toml version ([c743b20](https://github.com/sebastian-software/turndown-node/commit/c743b20271f65ea503ec25525f0bd6af9144990e))
* rename release.yml to publish.yml for npm trusted publishers ([1379956](https://github.com/sebastian-software/turndown-node/commit/1379956df13a99855715e8408fae071a57b114d9))

## [0.0.5](https://github.com/sebastian-software/turndown-node/compare/turndown-node-v0.0.4...turndown-node-v0.0.5) (2026-01-27)


### Bug Fixes

* use npm Trusted Publishers with provenance ([f1cd196](https://github.com/sebastian-software/turndown-node/commit/f1cd196d9a2de8368148089b80c181ff7932455e))

## [0.0.4](https://github.com/sebastian-software/turndown-node/compare/turndown-node-v0.0.3...turndown-node-v0.0.4) (2026-01-27)


### Bug Fixes

* exclude CHANGELOG.md from prettier checks ([544cd52](https://github.com/sebastian-software/turndown-node/commit/544cd529a88627d6a02c977a5c99ca452526a5d1))
* ignore lock file and upstream tests in prettier ([1844d09](https://github.com/sebastian-software/turndown-node/commit/1844d09b9f9383c876584fb5ae60dff4eb2bd65f))

## [0.0.3](https://github.com/sebastian-software/turndown-node/compare/turndown-node-v0.0.2...turndown-node-v0.0.3) (2026-01-27)


### Bug Fixes

* format changelog and run only parity tests in CI ([96012f2](https://github.com/sebastian-software/turndown-node/commit/96012f22257bfc68272995b7199064d2adba19f5))
* remove explicit pnpm version (use packageManager field) ([683d2e6](https://github.com/sebastian-software/turndown-node/commit/683d2e61a9fbf4abdac1b6b87ebb473aa2f985ff))
* use correct dtolnay/rust-toolchain action ([b1d94f3](https://github.com/sebastian-software/turndown-node/commit/b1d94f35d645787ddc6a849b93ce315bd763381d))

## [0.0.2](https://github.com/sebastian-software/turndown-node/compare/turndown-node-v0.0.1...turndown-node-v0.0.2) (2026-01-27)

### Features

- initial release ([69a6192](https://github.com/sebastian-software/turndown-node/commit/69a6192e27312d01ee0ec7c5eada708a19ac0d64))
