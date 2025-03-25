"""Provides commit message generation using ollama."""

from ollama import chat, ChatResponse

def generate_commit_message_stream(diff_text: str, model_name: str) -> str:
    """
    Generate a commit message using the specified model in ollama (streaming).

    :param diff_text: The staged diff text from the Git repository.
    :param model_name: The name of the model to be used in ollama.
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

    messages = [
        {
            "role": "user",
            "content": prompt_content
        }
    ]

    # Enable streaming: This returns a generator that yields chunks of response.
    stream = chat(model=model_name, messages=messages, stream=True)

    final_message = ""

    for chunk in stream:
        # Get the text content from the chunk
        text_chunk = chunk["message"]["content"]
        # Print the text chunk immediately so the user sees generation in real time
        print(text_chunk, end="", flush=True)
        final_message += text_chunk

    # Print a newline after streaming completes
    print()

    return final_message.strip()
