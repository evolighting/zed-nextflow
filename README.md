# Nextflow for Zed

Nextflow language support for [Zed](https://zed.dev/), backed by the official
Nextflow Tree-sitter grammar and language server.

## Features

- Tree-sitter highlighting for `.nf`, `.nf.test`, and `.config` files
- Bash highlighting inside `script:`, `shell:`, and `stub:` bodies
- Bracket matching, automatic indentation, code outline, and Vim text objects
- LSP completion, diagnostics, formatting, hover, navigation, references,
  rename, document symbols, and semantic tokens
- Separate `nextflow` and `nextflow-config` LSP document identifiers
- A shared `~/.nextflow/lsp/vXX.YY/` cache compatible with the official VS Code
  extension, plus an explicit cache check/update command

The language server requires Java 17 or later. Make sure `java` is available
on the shell `PATH` that Zed inherits.

## Install for development

1. Open Zed's Extensions view (`zed: extensions`).
2. Select **Install Dev Extension**.
3. Select this `zed-nextflow` directory.
4. Open `examples/main.nf` to exercise highlighting and LSP startup.

Zed compiles Rust extensions for `wasm32-wasip2` and builds the configured
Tree-sitter grammar during dev-extension installation. See the official
[developing extensions](https://zed.dev/docs/extensions/developing-extensions)
and [language extensions](https://zed.dev/docs/extensions/languages)
documentation for prerequisites and troubleshooting.

The extension defaults to the `26.04` language line and selects the greatest
stable patch JAR in `~/.nextflow/lsp/v26.04/`, using the same directory and
filename convention as the official VS Code extension. If the cache is empty,
the first LSP startup downloads the newest matching stable release. Existing
cached JARs are never deleted, so VS Code and Zed can safely reuse them.

The cache operations run in the worktree environment. The extension uses
PowerShell (`pwsh` or Windows PowerShell) on Windows and Bash on Linux/macOS.
Both implementations first reuse the greatest matching patch JAR already in
the VS Code-compatible cache; only an empty cache triggers a GitHub query and
download. In an SSH project, platform detection, the cache, Java executable,
and network access are all on the remote host.

## Check or update the shared LSP cache

The bundled command supports `check`, `update`, and `resolve`:

```sh
./scripts/nextflow-lsp-cache check 26.04
./scripts/nextflow-lsp-cache update 26.04
./scripts/nextflow-lsp-cache resolve 26.04
```

On Windows PowerShell:

```powershell
.\scripts\nextflow-lsp-cache.ps1 check 26.04
.\scripts\nextflow-lsp-cache.ps1 update 26.04
.\scripts\nextflow-lsp-cache.ps1 resolve 26.04
```

To make it available from any directory on a development machine:

```sh
install -Dm755 scripts/nextflow-lsp-cache ~/.local/bin/nextflow-lsp-cache
nextflow-lsp-cache check 26.04
nextflow-lsp-cache update 26.04
```

Downloads use a process-specific temporary file in the target cache directory,
verify the JAR/ZIP signature, and rename it atomically. Set `GITHUB_TOKEN` if
anonymous GitHub API rate limits are a concern. The optional
`NEXTFLOW_LSP_CACHE_DIR` environment variable overrides the cache root.

## Recommended Zed settings

Tree-sitter highlighting works without extra settings. Zed currently leaves
LSP semantic tokens disabled by default; use `combined` to layer them over the
Tree-sitter highlights:

```json
{
  "languages": {
    "Nextflow": {
      "format_on_save": "on",
      "semantic_tokens": "combined"
    },
    "Nextflow Config": {
      "format_on_save": "on",
      "semantic_tokens": "combined"
    }
  },
  "lsp": {
    "nextflow-language-server": {
      "settings": {
        "nextflow": {
          "languageVersion": "26.04",
          "completion": {
            "extended": true,
            "maxItems": 100
          },
          "errorReportingMode": "warnings",
          "formatting": {
            "harshilAlignment": false,
            "maheshForm": false,
            "sortDeclarations": false
          }
        }
      }
    }
  }
}
```

The extension supplies the same sensible workspace defaults as the VS Code
extension. User-provided values are recursively merged over those defaults and
the result is sent through `workspace/didChangeConfiguration`.
`nextflow.languageVersion` also selects the shared cache subdirectory and must
use a `YY.MM` value.

## Verify the language server

Open an `.nf` file and confirm that the language selector shows `Nextflow`.
Then enter this snippet and request completion after the dot:

```nextflow
workflow {
    log.
}
```

The official language server should offer `info`, `error`, and `warn`. Use
`dev: open language server logs` to inspect `Nextflow Language Server`; its
Server Info should show the selected Java executable and cached JAR. After a
source change, run `zed: rebuild dev extension`, followed by
`editor: restart language server`. With an SSH project, the process and cache
are on the remote host, while the logs viewer remains in the local Zed UI.

### Zed SSH session limitation

Zed currently has open upstream issues where extension-provided language
servers are not rebound to restored buffers after an SSH reconnect or an
extension upload. A characteristic log sequence is a language-server start
immediately followed by another `Loaded language server` entry, after which the
server process disappears and the LSP logs remain empty:

- [zed-industries/zed#31468](https://github.com/zed-industries/zed/issues/31468)
- [zed-industries/zed#60328](https://github.com/zed-industries/zed/issues/60328)

This occurs before the extension's language-server callback can run and cannot
currently be repaired by the extension itself. Wait for `Finished uploading
extension nextflow`, then close all restored Nextflow buffers and explicitly
open a different `.nf` file. If the remote workspace remains stale, create a
fresh remote project window rooted at the pipeline repository rather than
reconnecting to the restored window.

## Custom language-server command

The `binary` setting replaces the entire automatically managed Java/JAR
command. This is useful for a locally built server, a preview release, or a
Java executable that is not on `PATH`:

```json
{
  "lsp": {
    "nextflow-language-server": {
      "binary": {
        "path": "/absolute/path/to/java",
        "arguments": [
          "-jar",
          "/absolute/path/to/language-server-all.jar"
        ]
      }
    }
  }
}
```

You can also point `binary.path` at a native or wrapper executable and provide
its required `arguments` and `env`.

## File associations

The `.config` association matches the official VS Code extension. Because that
suffix is generic, override the association in your Zed `file_types` settings
if another language owns `.config` files in your projects.

## Development checks

```sh
cargo fmt --check
cargo check
cargo check --target wasm32-wasip2
bash -n scripts/nextflow-lsp-cache
scripts/nextflow-lsp-cache check 26.04
```

On Windows, run `.\scripts\nextflow-lsp-cache.ps1 check 26.04` from
PowerShell to validate the platform-specific cache manager.

Query files can be checked against the sibling grammar checkout used to create
this project:

```sh
tree-sitter query languages/nextflow/highlights.scm examples/main.nf
```

Before publishing under a different organization, update `authors` and
`repository` in `extension.toml`. Zed extensions are published through the
[`zed-industries/extensions`](https://github.com/zed-industries/extensions)
registry.

## Upstream projects

- [`nextflow-io/tree-sitter-nextflow`](https://github.com/nextflow-io/tree-sitter-nextflow)
- [`nextflow-io/language-server`](https://github.com/nextflow-io/language-server)
- [`nextflow-io/vscode-language-nextflow`](https://github.com/nextflow-io/vscode-language-nextflow)
