# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-17

### Added

- Initial release of Apinox schema-first API documentation generator
- CLI with 7 subcommands: `validate`, `build`, `watch`, `diff`, `migrate`, `import`, `sync`
- Schema format v1.0 with support for endpoints, auth schemes, environments, groups, changelog, error patterns
- 7 output format generators: Postman Collection v2.1, OpenAPI 3.1, Markdown docs, Scalar interactive HTML, Insomnia import v4, Hurl test scripts, curl shell scripts
- YAML/JSON parser with include support for modular schemas
- Schema validator with duplicate detection, ref checking, path-param matching, and soft warnings
- OpenAPI 3.x / Swagger 2.0 importer
- Schema diff and migration guide generator with changelog support
- Postman API sync (create/update collections)
- File watcher with debounce via `notify` crate
- Cross-platform builds: Linux x86_64, Linux ARM64, macOS x86_64, macOS ARM64, Windows x86_64
- GitHub Actions release workflow for automated binary distribution
- Landing page at apinox.denisetiya.site with installer scripts
- Build-all script for local multi-platform compilation
