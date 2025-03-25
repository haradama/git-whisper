import pytest
from git_whisper.generator import generate_commit_message_stream, CommitMessage

@pytest.mark.parametrize("diff_text", [
    "Some diff content",
    "",
    "Another diff example"
])
def test_generate_commit_message_stream(mocker, diff_text):
    """
    Test that generate_commit_message_stream correctly processes the streamed JSON
    and returns the formatted commit message. We'll mock the chat function so that
    no real call to Ollama is made.
    """

    # The chunk (JSON string) returned by the mock.
    # This JSON simulates CommitMessage(title="Add feature", changes=["Update README","Fix bug"]).
    mock_json = """{"title": "Add feature","changes":["Update README","Fix bug"]}"""
    mock_stream = iter([
        {"message": {"content": mock_json}}
    ])

    # Use mocker to mock the chat_func.
    mock_chat_func = mocker.MagicMock(return_value=mock_stream)

    result = generate_commit_message_stream(
        diff_text=diff_text,
        model_name="test-model",
        chat_func=mock_chat_func
    )

    # Verify that the JSON chunk is correctly processed as a CommitMessage
    # and that the function returns the result in the specified format.
    assert "Add feature" in result
    assert "- Update README" in result
    assert "- Fix bug" in result

    # Verify that the mock was called once.
    mock_chat_func.assert_called_once()
