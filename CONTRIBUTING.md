# Plugin development

## Development

Rust is pinned in `rust-toolchain.toml`. The SDK Git revision in `Cargo.toml` and author CLI
revision in `.github/workflows/checks.yml` must agree, with a committed `Cargo.lock`.
Install the CLI directly from that Git revision; no sibling checkout is required.

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings
cargo install synra-plugin-cli --git https://github.com/synra-rs/synra-core.git --rev ffec0a2dd3833346bccc6f75bb49447c4e2cc9eb --locked
pnpm install --frozen-lockfile
pnpm run lint
pnpm run check
pnpm test
synra-plugin build
synra-plugin verify --dir dist
git diff --exit-code -- Cargo.lock
```

The CLI converts the SDK-generated module to a real component and verifies its binary ABI
and manifest. `plugin.wasm` and Cargo output are ignored. CI checks development artifacts
without releasing or publishing packages.

## Hotkey contract

- The phone invokes `hotkey.execute` with an explicitly selected target device and a built-in
  shortcut ID. The action never accepts an arbitrary key chord.
- The plugin sends the shortcut ID over `_plugin.@synra-plugin/hotkey.execute` and the computer
  maps that ID to its fixed chord before calling `platform:simulate-keys`.
- The computer replies over `_plugin.@synra-plugin/hotkey.response`; the phone polls the bounded
  response store through `hotkey.response-poll`.
- Every request is bounded by a validated device ID, request ID and shortcut ID. Unknown IDs are
  rejected before any host call.
- Native key simulation, device delivery and permission prompts require acceptance on a real
  Synra phone and computer; unit tests and `synra-plugin verify` cover different layers.

## UI verification

The Vue tests cover shortcut labels, fixed action IDs, target decoding, column switching and
order normalization. The UI keeps fixed shortcut definitions and persists only column count and
order. The drag interaction should be exercised on a touch device in addition to the automated
pointer tests.

See the [Hotkey design documents](./ai-docs/design-docs) and the [official plugin contract](https://github.com/synra-rs/design-docs/blob/main/29-official-plugins.md).
