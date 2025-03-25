# git-whisper

`git-whisper` is a local commit message generator tool that uses
[ollama](https://github.com/jmorganca/ollama).  
Unlike cloud-based commit message generation tools—which can be costly for small teams or prohibited by organizational policies—`git-whisper` runs **entirely on your local environment**, making it a practical choice when budgets or security rules are tight.

This tool streams LLM-generated commit messages based on your staged diff and lets you interactively decide whether to accept or reject the suggestions.

## Prerequisites

- **Ollama server must be running**  
  Make sure you have [Ollama](https://github.com/jmorganca/ollama) installed and started locally:

  ```bash
  ollama serve
  ```

If Ollama is not running, `git whisper` will be unable to generate a commit message.

## Installation

```bash
pip install git-whisper
```

## Usage

1. Make changes to your files and stage them:

   ```bash
   git add <files>
   ```

2. Run:

   ```bash
   git whisper
   ```

   - The tool reads your staged diff and uses the locally running Ollama server to generate a commit message.
   - The generated message is streamed in real time to your terminal.
   - After the generation finishes, you'll see `Accept this message? (Y/n)`.
     - Press `Enter` or type `Y` to accept. The message is automatically committed with `git commit -m "<generated message>"`.
     - Type `n` to cancel. No commit will be made.

3. **Optional**: One-liner usage

   ```bash
   git commit -m "$(git whisper)"
   ```

   However, keep in mind that the default setup is interactive and will prompt you before finalizing the commit.

## Configuration

You can specify a custom model in your local Git config to override the default model. For example:

```bash
git config git-whisper.model llama3
```

If you do not specify a model, `"llama3"` is used by default.

## License

This project is provided under the MIT License. See [LICENSE](./LICENSE) for details.

