"""CLI entry point for the git-whisper tool."""

import sys
from git import Repo, InvalidGitRepositoryError, NoSuchPathError
from git_whisper.generator import generate_commit_message_stream

def main():
    """
    Entry point for the git-whisper tool.
    This function reads the staged diff and generates a commit message
    by calling the local LLM via ollama in a streaming manner.
    After generation, the user can choose (Y/n) to accept or reject the message.
    """

    try:
        repo = Repo(".", search_parent_directories=True)
    except (InvalidGitRepositoryError, NoSuchPathError):
        print("Error: Not a valid Git repository.")
        sys.exit(1)

    # Check if there has been any commit yet. If not, treat it as 'initial commit'.
    is_initial_commit = not repo.head.is_valid()

    # Read user's configured model from git config if it exists; otherwise set a default.
    model_name = "llama3"
    config_reader = repo.config_reader()
    try:
        model_name = config_reader.get_value("git-whisper", "model")
    except Exception:
        pass

    if is_initial_commit:
        diff_text = "[Initial commit detected; no diff available]"
    else:
        diff_text = repo.git.execute(["git", "diff", "--staged"])
        if not diff_text:
            print("Error: No staged diff found; cannot generate a commit message")
            sys.exit(1)

    print("Generating commit message (streaming). Please wait...\n")
    commit_message = generate_commit_message_stream(diff_text, model_name)

    print("\n[Commit message preview above]")
    while True:
        choice = input("Accept this message? (Y/n): ").strip().lower()
        if choice == "" or choice == "y":
            repo.git.commit(m=commit_message)
            print("\nAccepted commit message:\n")
            sys.exit(0)
        elif choice == "n":
            sys.exit(0)
        else:
            print("Invalid choice. Please enter 'Y' or 'n'.\n")
