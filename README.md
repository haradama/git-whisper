# git-whisper

`git-whisper` is a local commit message generator tool using
[ollama](https://github.com/jmorganca/ollama).
This tool streams LLM-generated commit messages based on your staged diff and allows you to accept or reject them interactively.

## Installation

1. Clone or download the repository.
2. Inside the project folder, run

   ```bash
   pip install .
   ```

   Or, if you are using [Poetry](https://python-poetry.org/):

   ```bash
   poetry build
   pip install dist/git_whisper-0.1.0-py3-none-any.whl
   ```

In the future, if this package is published on PyPI, you could simply do

```bash
pip install git-whisper
```

## Usage

Make changes to your files and stage them

```bash
git add <files>
```

Run

```bash
git whisper
```

You can also do

```bash
git commit -m "$(git whisper)"
```

- The tool will read your staged diff and use a locally running LLM (via `ollama`) to generate a commit message.
- The generated message will appear on your terminal as it streams in real time.
- Once generation is complete, you'll be prompted with: `Accept this message? (Y/n)`
  - Press `Enter` or type `Y` to accept. The message is automatically committed with `git commit -m "<generated message>"`.
  - Type `n` to cancel. No commit will be made.

## Configuration

You can specify a model in your local Git config to override the default model. For example:

```bash
git config git-whisper.model llama3
```

If you do not specify a model, the tool defaults to `"llama3"`.

## License

MIT License. See [LICENSE](./LICENSE) for details.
