# git-whisper

**git-whisper** is a commit-message generator powered by an Ollama-served
LLM.  
It streams a concise commit message for your staged diff, shows it in real time,
then asks whether to use it for `git commit`.  
Everything runs **entirely on your machine**—no source code or diff ever leaves your
laptop.

## Key features

| Feature | Description |
|---------|-------------|
| **Streamed preview** | Tokens appear as they’re generated; once finished the raw JSON is erased and replaced with a formatted preview. |
| **Config-driven** | `git config git-whisper.model` and `git-whisper.prompt` override the model and prompt per-repo or globally. |
| **100 % local** | Uses your local Ollama server; works offline or behind strict firewalls. |
| **Multilingual** | Write the prompt in any language (e.g., Japanese) and the commit message follows suit. |

## Prerequisites

* **Ollama server running** (≥ 0.1.34 with `/api/chat` & `format` support).

```bash
ollama run llama3
````

## Installation

```bash
cargo install --git https://github.com/haradama/git-whisper --locked
```

Build artifacts live in `~/.cargo/bin/git-whisper`.

## Quick start

```bash
# after editing files
git add .

# generate message, decide, and optionally commit
git whisper        # interactive mode
```

## Configuration

### Select an LLM model

```bash
git config --global git-whisper.model llama3
```

(default: `llama3`)

### Customize the prompt

Tell git-whisper how to ask the model.

Use `{diff}` as a placeholder for the staged diff; any language works:

```bash
git config --global git-whisper.prompt \
"以下の Git diff を読み、日本語でコミットメッセージ (title と changes) を JSON で返してください。\n\n{diff}"
```

Per-repository overrides:

```bash
git config git-whisper.model phi3
git config git-whisper.prompt "$(cat .prompt/git_whisper.txt)"
```

## License

`git-whisper` is released under the MIT License.

See [LICENSE](./LICENSE) for full text.
