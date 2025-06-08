//! ストリーミングで JSON を受け取りつつ逐次表示し、
//! 最後にログをクリアして整形済みメッセージを出力する。

use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::error::Error;
use std::io::{self, Write};

#[derive(Debug, Deserialize)]
pub struct CommitMessage {
    pub title: String,
    pub changes: Vec<String>,
}

/// diff・モデル名・任意プロンプトを受け取りコミットメッセージを生成
///
/// * `prompt_tpl`:
///   - `None`         → デフォルト英語プロンプトを使用
///   - `Some(str)`    → 文字列内の `{diff}` を Git diff で置換して使用
pub async fn generate_commit_message_stream(
    diff_text: &str,
    model_name: &str,
    prompt_tpl: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    /* ───── プロンプト合成 ───── */
    const DEFAULT_PROMPT: &str = r#"
You are a professional software engineer that generates concise, clear commit messages.
Please generate a commit message in the following format and follow these rules:
1. Do not include any additional text beyond the commit message itself.
2. The commit message must consist of exactly two parts:
   - A short, descriptive title on the first line (≈50 characters).
   - A bullet-point list of changes made, each on its own line.

Given the following Git diff, please provide a short commit message in JSON with keys
`title` (string) and `changes` (array of strings). Output the keys in that order.

DIFF:
{diff}
"#;

    let tpl = prompt_tpl.unwrap_or(DEFAULT_PROMPT);
    let prompt_content = if tpl.contains("{diff}") {
        tpl.replace("{diff}", diff_text)
    } else {
        format!("{tpl}\n\nDIFF:\n{diff_text}")
    };

    /* ───── JSON スキーマ (Ollama format) ───── */
    let schema: Value = json!({
        "type": "object",
        "properties": {
            "title":   { "type": "string" },
            "changes": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["title", "changes"]
    });

    /* ───── リクエスト送信 ───── */
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

    /* ───── ストリーム処理 ───── */
    let mut raw_json   = String::new();
    let mut remainder  = String::new();
    let mut line_count = 0usize;
    let mut stream     = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        remainder.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(idx) = remainder.find('\n') {
            let mut line: String = remainder.drain(..=idx).collect();
            line.pop();                                  // '\n'
            handle_line(&line, &mut raw_json, &mut line_count)?;
        }
    }
    if !remainder.trim().is_empty() {
        handle_line(&remainder, &mut raw_json, &mut line_count)?;
    }

    /* ───── JSON → 構造体 ───── */
    let parsed: CommitMessage = serde_json::from_str(raw_json.trim())?;

    /* ───── ストリームをクリア ───── */
    clear_lines(line_count)?;

    /* ───── 整形メッセージ生成 ───── */
    let mut formatted = parsed.title.trim().to_string();
    formatted.push_str("\n\n");
    for ch in parsed.changes {
        formatted.push_str("- ");
        formatted.push_str(ch.trim_start_matches("- ").trim());
        formatted.push('\n');
    }
    println!("{formatted}");

    Ok(formatted)
}

/* ───────────────────────────────────────────── */
/* 補助関数                                      */
/* ───────────────────────────────────────────── */

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
