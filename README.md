# git-whisper

![Demo](demo.gif)

**git-whisper** is a commit-message generator powered by an Ollama-served
LLM.  
It streams a concise commit message for your staged diff, shows it in real time,
then asks whether to use it for `git commit`.  
Everything runs **entirely on your machine**—no source code or diff ever leaves your
laptop.

## Key features

| Feature                      | Description                                                                                                     |
| ---------------------------- | --------------------------------------------------------------------------------------------------------------- |
| **Streamed preview**         | Tokens appear as they're generated; once finished the raw JSON is erased and replaced with a formatted preview. |
| **Interactive accept/edit**  | After preview, choose to accept, reject, or edit the message before committing.                                 |
| **Intent hint (`--intent`)** | Provide the “why” behind the change when it can't be inferred from a diff.                                      |
| **Config-driven**            | `git config git-whisper.model` and `git-whisper.prompt` override the model and prompt per-repo or globally.     |
| **100 % local**              | Uses your local Ollama server; works offline or behind strict firewalls.                                        |
| **Multilingual**             | Write the prompt in any language (e.g., Japanese) and the commit message follows suit.                          |

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

### Provide an intent hint

A good commit message explains **why** the change is made.
However, a Git diff alone often lacks that context.

Use `--intent` to supply a short hint describing the intent:

```bash
git whisper --intent "To make 'cargo clippy -- -D warnings' pass"
```

### Edit before committing

After the preview, you'll be prompted:

```text
Accept this message? (Y/n/e to edit):
```

* `Y` (or empty) commits with the generated message
* `n` aborts
* `e` opens your editor to modify the message before accepting

To choose an editor, set `$EDITOR` (or `$VISUAL` if supported in your environment), for example:

```bash
export EDITOR="code --wait"
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

#### Prompt placeholders

When using a custom prompt, you can use these placeholders:

* `{diff}`: the staged Git diff (required for best results)
* `{intent}`: the optional intent hint from `--intent` (may be empty)

If `{intent}` is not present in your prompt, git-whisper will append the intent hint to the prompt when provided.

## License

`git-whisper` is released under the MIT License.

See [LICENSE](./LICENSE) for full text.
