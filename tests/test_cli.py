import pytest
import sys
from unittest.mock import MagicMock, patch
from git_whisper.cli import main
from git import InvalidGitRepositoryError, NoSuchPathError

def test_main_valid_repo_accept(mocker):
    """
    Covers the path: valid repo, HEAD is valid -> is_initial_commit=False,
    user inputs 'y' (accept), so the code calls `repo.git.execute(...)`.
    Lines 29-30 should be covered.
    """

    mock_repo = mocker.MagicMock()
    mock_repo.head.is_valid.return_value = True  # => is_initial_commit=False
    mock_repo.git.execute.return_value = "fake diff"
    mock_repo.git.commit = MagicMock()

    # Repo(...) constructor returns mock_repo
    mocker.patch("git_whisper.cli.Repo", return_value=mock_repo)

    # Simulate user pressing 'y'
    mocker.patch("builtins.input", return_value="y")

    # Mock generator function so it doesn't actually call Ollama
    mocker.patch("git_whisper.cli.generate_commit_message_stream", return_value="Fake Message")

    with pytest.raises(SystemExit) as e:
        main()

    # exit code = 0
    assert e.value.code == 0
    # diff_text should have come from repo.git.execute
    mock_repo.git.execute.assert_called_once_with(["git", "diff", "--staged"])
    # commit should be called
    mock_repo.git.commit.assert_called_once_with(m="Fake Message")


def test_main_valid_repo_reject(mocker):
    """
    Covers the path: valid repo, HEAD is valid -> is_initial_commit=False,
    user inputs 'n' (reject).
    """

    mock_repo = mocker.MagicMock()
    mock_repo.head.is_valid.return_value = True
    mock_repo.git.execute.return_value = "fake diff"
    mock_repo.git.commit = MagicMock()

    mocker.patch("git_whisper.cli.Repo", return_value=mock_repo)
    mocker.patch("builtins.input", return_value="n")
    mocker.patch("git_whisper.cli.generate_commit_message_stream", return_value="Fake Message")

    with pytest.raises(SystemExit) as e:
        main()

    # exit code = 1
    assert e.value.code == 1
    mock_repo.git.execute.assert_called_once()
    mock_repo.git.commit.assert_not_called()


def test_main_invalid_repo(mocker):
    """
    Covers the path: Repo(...) raises InvalidGitRepositoryError,
    so we catch it and sys.exit(1).
    This ensures we cover the 'except' block and subsequent lines in that block.
    """

    # Make Repo(...) raise
    mocker.patch("git_whisper.cli.Repo", side_effect=InvalidGitRepositoryError)

    with pytest.raises(SystemExit) as e:
        main()
    # exit code = 1
    assert e.value.code == 1


def test_main_no_commit_y(mocker):
    """
    Covers the path: HEAD is invalid -> is_initial_commit=True,
    user inputs 'y' -> commit with message "[Initial commit detected; no diff available]"
    """

    mock_repo = mocker.MagicMock()
    mock_repo.head.is_valid.return_value = False  # => is_initial_commit=True
    mock_repo.git.execute = MagicMock()
    mock_repo.git.commit = MagicMock()

    mocker.patch("git_whisper.cli.Repo", return_value=mock_repo)
    mocker.patch("builtins.input", return_value="y")
    mocker.patch("git_whisper.cli.generate_commit_message_stream",
                 return_value="Fake Message for initial commit")

    with pytest.raises(SystemExit) as e:
        main()

    assert e.value.code == 0
    # Because is_initial_commit=True, it won't call `repo.git.execute(...)`
    mock_repo.git.execute.assert_not_called()
    mock_repo.git.commit.assert_called_once_with(m="Fake Message for initial commit")

def test_main_config_reader_exception(mocker):
    """
    Test that if config_reader.get_value raises an exception,
    the code goes through `except Exception: pass`
    (covers that block).
    """

    mock_repo = mocker.MagicMock()
    mock_repo.head.is_valid.return_value = True  # not initial commit
    mock_repo.git.execute.return_value = "fake diff"
    mock_repo.git.commit = MagicMock()

    # When we call config_reader.get_value, raise an exception
    mock_config = mocker.MagicMock()
    mock_config.get_value.side_effect = RuntimeError("Test exception")

    # Make repo.config_reader() return the mock_config
    mock_repo.config_reader.return_value = mock_config

    # Patch Repo(...) to return our mock_repo
    mocker.patch("git_whisper.cli.Repo", return_value=mock_repo)

    # We'll have user input 'n' => reject => exit(1)
    mocker.patch("builtins.input", return_value="n")

    # Mock generator so it doesn't call Ollama
    mocker.patch("git_whisper.cli.generate_commit_message_stream", return_value="Fake Message")

    with pytest.raises(SystemExit) as e:
        main()
    # exit code = 1
    assert e.value.code == 1

    # We confirm that we got here without error
    # The exception should have been silently passed
    mock_config.get_value.assert_called_once_with("git-whisper", "model")


@patch("builtins.input", side_effect=["z", "n"])
def test_main_invalid_choice(mock_input, mocker):
    """
    Covers the 'else' branch for the user input (invalid choice).
    First input: "z" (invalid) => triggers the else block
    Second input: "n" => exit(1) and end the loop
    """

    mock_repo = mocker.MagicMock()
    mock_repo.head.is_valid.return_value = True
    mock_repo.git.execute.return_value = "fake diff"
    mock_repo.git.commit = MagicMock()

    mocker.patch("git_whisper.cli.Repo", return_value=mock_repo)
    mocker.patch("git_whisper.cli.generate_commit_message_stream", return_value="Fake Message")

    with pytest.raises(SystemExit) as e:
        main()

    # Because second input is 'n'
    assert e.value.code == 1

    # The invalid choice "z" should have triggered the else block,
    # Then "n" ended with sys.exit(1).
    mock_repo.git.commit.assert_not_called()
