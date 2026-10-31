# WordGlow

WordGlow is a local desktop writing assistant built with Rust and Dioxus.

It provides:

- Multiple documents
- Automatic local saving
- Real-time offline grammar and spelling checks
- US English and UK English options
- Optional AI suggestions
- Grammar lessons for detected issues

## Local checking

The normal grammar check runs on your computer. It does not need an internet connection and does not send your document anywhere.

It uses:

- [Harper](https://github.com/elijah-potter/harper) for grammar rules
- [typos](https://github.com/crate-ci/typos) for spelling corrections
- A few application-specific common-error rules

The local checker is fast and private, but it cannot detect every possible writing problem.

## AI checking

AI checking is optional. The app supports:

- Ollama for local models
- OpenAI
- Anthropic
- Google

For Ollama:

1. Install and start [Ollama](https://ollama.com/).
2. Download a model, for example:

   ```powershell
   ollama pull qwen2.5:3b
   ```

3. Open the app settings and select **Ollama**.
4. Select the installed model.
5. Click **Check with AI**.

Ollama requests stay on your computer. Requests to OpenAI, Anthropic, or Google send the document to the selected provider. API keys are kept in memory and are not saved.

## Run from source

Install Rust and the [Dioxus CLI](https://dioxuslabs.com/learn/0.6/getting_started/):

```powershell
cargo install dioxus-cli --locked
```

Start the desktop app from the repository root:

```powershell
dx serve --platform desktop --manifest-path app/Cargo.toml
```

Build a release version:

```powershell
dx build --platform desktop --release --manifest-path app/Cargo.toml
```

## Test and format

Run these commands from the repository root:

```powershell
cargo fmt --manifest-path app/Cargo.toml -- --check
cargo check --manifest-path app/Cargo.toml
cargo test --manifest-path app/Cargo.toml
```

## Documents and settings

Documents and provider settings are saved locally in:

```text
%APPDATA%\Grammar Check\documents.json
```

On other operating systems, the file is stored in the platform's application-data directory.

Edits are saved automatically after a short pause and when the editor loses focus. The save button can be used to save immediately.

## Builds

GitHub Actions tests and builds the app for Windows, macOS, and Linux. Windows builds include a portable ZIP and an Inno Setup installer. The installer installs
the executable and its required assets together and creates Start menu and optional desktop
shortcuts. Builds are unsigned, and macOS notarization is not configured.
