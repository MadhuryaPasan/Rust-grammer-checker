# Grammar Check

A native Dioxus desktop writing checker with a multi-document editor, automatic local saving, offline common-error checks, optional AI suggestions, and US/UK English spelling preferences.

## Run the desktop app

From `app`:

```powershell
dx serve --platform desktop
```

Or build a desktop bundle:

```powershell
dx build --platform desktop
```

Run the test suite with:

```powershell
cargo test --manifest-path app/Cargo.toml
```

## Documents and saving

Documents and provider preferences are stored locally in `documents.json` under the user's application-data folder (`%APPDATA%\Grammar Check` on Windows). Edits save automatically after a short pause and when the editor loses focus. Use the save button in the editor toolbar to save immediately. API keys are not persisted.

## English variants and offline checking

Choose US English or UK English in the editor toolbar. The built-in checker applies a small list of spelling differences for that variant, plus a small list of common typos and grammar patterns.

The offline checker is **not AI** and does not use a third-party spelling/grammar library or full dictionary. It matches known text patterns and gives fixed corrections. It is fast, private, and works without internet or a model, but cannot understand context and will miss many mistakes. The UI labels this as a basic offline check.

For context-aware suggestions, choose **Check with AI** to send the current document to the selected model. Ollama's `/api/tags` endpoint discovers local models. OpenAI, Anthropic, and Google model-list and text-check APIs are also supported. AI requests send document text directly to the configured provider; local Ollama keeps the request on your machine. API keys are kept in memory only.

## Cross-platform builds

The GitHub Actions workflow runs tests and builds the desktop app on Windows, macOS, and Linux for pushes to `main`/`master`, pull requests, or manual dispatch. Downloadable build artifacts are uploaded for each platform. Builds are unsigned; macOS notarization and signed installers require platform certificates and are not configured.
