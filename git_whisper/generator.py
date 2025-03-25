"""Provides commit message generation using ollama."""

from typing import List, Callable, Any, Iterator

from ollama import chat
from pydantic import BaseModel

class CommitMessage(BaseModel):
    title: str
    changes: List[str]

def generate_commit_message_stream(
    diff_text: str,
    model_name: str,
    chat_func: Callable[..., Iterator] = chat
) -> str:
    """
    Generate a commit message using the specified model in ollama (streaming).
    This function supports dependency injection of chat_func for testing.

    :param diff_text: The staged diff text from the Git repository.
    :param model_name: The name of the model to be used in ollama.
    :param chat_func: (Optional) A function compatible with ollama.chat for mock testing.
    :return: A generated commit message as a string.
    """

    prompt_content = f"""You are an AI that generates concise, clear commit messages.
Please generate a commit message in the following format and follow these rules:
1. Do not include any additional text beyond the commit message itself.
2. The commit message must consist of exactly two parts:
  - A short, descriptive title on the first line (approximately 50 characters).
  - A bullet-point list of changes made, each on its own line.

Given the following Git diff, please provide a short commit message in English.

DIFF:
{diff_text}
"""

    # Use the injected chat_func instead of directly calling ollama.chat
    stream = chat_func(
        model=model_name,
        messages=[
            {
                "role": "user",
                "content": prompt_content
            }
        ],
        format=CommitMessage.model_json_schema(),
        stream=True
    )

    response = ""
    for chunk in stream:
        text_chunk = chunk["message"]["content"]
        # Print streaming to stdout (we still do this, but it's fine for test coverage)
        print(text_chunk, end="", flush=True)
        response += text_chunk

    print()  # final newline

    generated_message = CommitMessage.model_validate_json(response)
    final_message = f"""
{generated_message.title}

{"".join(f"- {change}\n" for change in generated_message.changes)}
"""
    return final_message
