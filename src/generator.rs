use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use std::error::Error;
use std::io::{self, Write};

#[derive(Debug, Deserialize)]
pub struct CommitMessage {
    pub commit_type: String,
    pub title: String,
    pub changes: Vec<String>,
}

pub async fn generate_commit_message_stream(
    diff_text: &str,
    model_name: &str,
    prompt_tpl: Option<&str>,
    intent_hint: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    const DEFAULT_PROMPT: &str = r#"
You are a professional software engineer that writes high-quality commit messages.

Core principle:
- A good commit message explains intent (WHY) and the goal/outcome, not a line-by-line description of the diff.

Input:
- You will receive a Git diff and (optionally) an INTENT_HINT written by the user.
- If INTENT_HINT is present, treat it as the primary source of truth for "why".

Output requirements:
1. Output ONLY a JSON object with keys: `commit_type`, `title`, `changes`.
2. The final commit message will be rendered as:
   - "{commit_type}: {title}" on the first line
   - blank line
   - bullet list from `changes`, each item becomes "- {item}"

Commit message quality rules:
- Title MUST state the intent/outcome in ~50 chars (not "update X", not "refactor code").
  Prefer patterns like:
  - "Make <X> pass <Y>"
  - "Prevent <bad thing> in <context>"
  - "Enable <capability> for <reason>"
- Each bullet in `changes` MUST include intent language (use at least one of: "to", "so that", "because", "in order to").
  Bad: "Rename function", "Add check", "Update config"
  Good: "Rename X to clarify intent so that usage is unambiguous"
- Do NOT mechanically narrate the diff. Mention "what" only when necessary to support the "why".
- If intent cannot be inferred from the diff and INTENT_HINT is empty, write a cautious, general intent
  (e.g., maintainability, correctness, performance, developer experience) rather than listing file edits.

Classification:
`commit_type` must be one of:
fix, feat, docs, style, refactor, perf, test, build, ci, chore.

INTENT_HINT (optional, may be empty):
{intent}

DIFF:
{diff}
"#;

    let tpl = prompt_tpl.unwrap_or(DEFAULT_PROMPT);

    let mut prompt_content = if tpl.contains("{diff}") {
        tpl.replace("{diff}", diff_text)
    } else {
        format!("{tpl}\n\nDIFF:\n{diff_text}")
    };

    if prompt_content.contains("{intent}") {
        prompt_content = prompt_content.replace("{intent}", intent_hint.unwrap_or(""));
    } else if let Some(hint) = intent_hint {
        if !hint.trim().is_empty() {
            prompt_content.push_str("\n\nINTENT_HINT:\n");
            prompt_content.push_str(hint.trim());
            prompt_content.push_str(
                "\n\nGuidance: reflect this intent in the title and bullets; do not just restate the diff.\n",
            );
        }
    }

    let schema: Value = json!({
        "type": "object",
        "properties": {
            "commit_type": {
                "type": "string",
                "enum": ["fix","feat","docs","style","refactor","perf","test","build","ci","chore"]
            },
            "title":   { "type": "string" },
            "changes": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["commit_type","title","changes"]
    });

    let body = json!({
        "model": model_name,
        "messages": [{ "role": "user", "content": prompt_content }],
        "stream": true,
        "format": schema
    });

    let resp = Client::new()
        .post("http://localhost:11434/api/chat")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(format!("Ollama returned HTTP {}", resp.status()).into());
    }

    let mut raw_json = String::new();
    let mut remainder = String::new();
    let mut line_count = 0usize;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        remainder.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(idx) = remainder.find('\n') {
            let mut line: String = remainder.drain(..=idx).collect();
            line.pop();
            handle_line(&line, &mut raw_json, &mut line_count)?;
        }
    }
    if !remainder.trim().is_empty() {
        handle_line(&remainder, &mut raw_json, &mut line_count)?;
    }

    let parsed: CommitMessage = serde_json::from_str(raw_json.trim())?;

    clear_lines(line_count)?;

    let mut formatted = format!("{}: {}", parsed.commit_type, parsed.title.trim());
    formatted.push_str("\n\n");
    for ch in parsed.changes {
        formatted.push_str("- ");
        formatted.push_str(ch.trim_start_matches("- ").trim());
        formatted.push('\n');
    }
    println!("{formatted}");

    Ok(formatted)
}

fn handle_line(
    line: &str,
    raw_json: &mut String,
    line_count: &mut usize,
) -> Result<(), serde_json::Error> {
    let v: Value = serde_json::from_str(line)?;
    if let Some(text) = v
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
    {
        print!("{text}");
        io::stdout().flush().ok();
        *line_count += text.matches('\n').count();
        raw_json.push_str(text);
    }
    Ok(())
}

fn clear_lines(line_count: usize) -> io::Result<()> {
    if line_count == 0 {
        print!("\r\x1b[2K\r");
        return io::stdout().flush();
    }
    print!("\r");
    for _ in 0..line_count {
        print!("\x1b[2K\x1b[1A");
    }
    print!("\x1b[2K\r");
    io::stdout().flush()
}
