# Contributing

Please read [AGENTS.md](AGENTS.md), the [MVP](docs/MVP.md) and [Provider guide](docs/PROVIDERS.md). Start with an issue for major changes; keep PRs focused. Use stable Rust and pinned dependencies once the workspace is initialized. Run applicable format/lint/test/build tasks, provide platform-specific results for tray/credential changes, and document any unverified behavior.

Provider submissions require evidence of permitted access, sanitized fixtures, normalization tests, error mapping and a capability matrix update. Never upload account responses, API keys, cookies, OAuth tokens, private URLs or personal data. Do not bypass provider restrictions or add automatic paid fallback.

Use clear commit messages (e.g. `feat(monitor): add demo quota card`). Update docs and changelog with user-facing changes. Third-party code and assets must comply with their licenses. By contributing, you agree to license your contributions under the project's MIT license.
