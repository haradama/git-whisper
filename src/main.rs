use git_whisper::generator::generate_commit_message_stream;
use std::{
    io::{self, Write},
    process::Command,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* ───── Git リポジトリ確認 ───── */
    let repo = match git2::Repository::discover(".") {
        Ok(r) => r,
        Err(_) => {
            eprintln!("Error: Not a valid Git repository.");
            std::process::exit(1);
        }
    };
    let is_initial = repo.is_empty().unwrap_or(false) || repo.head().is_err();

    /* ───── git config から model / prompt を取得 ───── */
    let mut model  = "llama3".to_string();
    let mut prompt = None;                       // Option<String>
    if let Ok(cfg) = repo.config() {
        if let Ok(m) = cfg.get_string("git-whisper.model") {
            model = m;
        }
        if let Ok(p) = cfg.get_string("git-whisper.prompt") {
            prompt = Some(p);
        }
    }

    /* ───── ステージ済み diff 取得 ───── */
    let diff_text = if is_initial {
        "[Initial commit detected; no diff available]".to_string()
    } else {
        let out = Command::new("git").args(["diff", "--staged"]).output()?;
        let diff = String::from_utf8_lossy(&out.stdout).to_string();
        if diff.is_empty() {
            eprintln!("Error: No staged diff found; cannot generate a commit message");
            std::process::exit(1);
        }
        diff
    };

    println!("Generating commit message (streaming). Please wait…\n");

    /* ───── 生成 ───── */
    let commit_msg =
        generate_commit_message_stream(&diff_text, &model, prompt.as_deref()).await?;

    println!("\n[Commit message preview above]");
    loop {
        print!("Accept this message? (Y/n): ");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        match buf.trim().to_lowercase().as_str() {
            "" | "y" => {
                Command::new("git").args(["commit", "-m", &commit_msg]).status()?;
                println!("\nAccepted commit message:\n{commit_msg}");
                break;
            }
            "n" => break,
            _ => println!("Invalid choice. Please enter 'Y' or 'n'.\n"),
        }
    }
    Ok(())
}
