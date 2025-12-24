use git_whisper::generator::generate_commit_message_stream;
use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut intent: Option<String> = None;
    let mut it = env::args().skip(1);

    while let Some(arg) = it.next() {
        if arg == "--intent" {
            let v = it.next().unwrap_or_else(|| {
                eprintln!("Error: --intent requires a value (e.g. --intent \"...\" )");
                std::process::exit(2);
            });
            intent = Some(v);
        } else if let Some(v) = arg.strip_prefix("--intent=") {
            intent = Some(v.to_string());
        } else if arg == "-h" || arg == "--help" {
            println!(
                "Usage: git-whisper [--intent \"...\"]\n\n  --intent  Provide a hint describing WHY this change is made."
            );
            return Ok(());
        }
    }

    let repo = match git2::Repository::discover(".") {
        Ok(r) => r,
        Err(_) => {
            eprintln!("Error: Not a valid Git repository.");
            std::process::exit(1);
        }
    };
    let is_initial = repo.is_empty().unwrap_or(false) || repo.head().is_err();

    let mut model = "llama3".to_string();
    let mut prompt = None;
    if let Ok(cfg) = repo.config() {
        if let Ok(m) = cfg.get_string("git-whisper.model") {
            model = m;
        }
        if let Ok(p) = cfg.get_string("git-whisper.prompt") {
            prompt = Some(p);
        }
    }

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

    let mut commit_msg =
        generate_commit_message_stream(&diff_text, &model, prompt.as_deref(), intent.as_deref())
            .await?;

    println!("\n[Commit message preview above]");

    loop {
        print!("Accept this message? (Y/n/e to edit): ");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;

        match buf.trim().to_lowercase().as_str() {
            "" | "y" => {
                git_commit_with_message(&commit_msg)?;
                println!("\nAccepted commit message:\n{commit_msg}");
                break;
            }
            "n" => break,
            "e" => match edit_in_editor(&commit_msg) {
                Ok(edited) => {
                    commit_msg = edited.trim_end().to_string();
                    println!("\n[Edited commit message preview]\n\n{commit_msg}\n");
                }
                Err(e) => {
                    eprintln!("Failed to open editor: {e}");
                    eprintln!("Tip: set $EDITOR (e.g. export EDITOR=\"code --wait\")");
                }
            },
            _ => println!("Invalid choice. Please enter 'Y', 'n', or 'e'.\n"),
        }
    }

    Ok(())
}

fn git_commit_with_message(commit_msg: &str) -> io::Result<()> {
    let mut lines = commit_msg.lines();

    let subject = lines.next().unwrap_or("").trim().to_string();
    if subject.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Commit subject (first line) is empty",
        ));
    }

    let mut rest: Vec<&str> = lines.collect();
    if rest.first().map(|s| s.trim().is_empty()).unwrap_or(false) {
        rest.remove(0);
    }
    let body = rest.join("\n");

    let mut cmd = Command::new("git");
    cmd.arg("commit").arg("-m").arg(&subject);
    if !body.trim().is_empty() {
        cmd.arg("-m").arg(&body);
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "git commit failed with status: {status}"
        )));
    }
    Ok(())
}

fn edit_in_editor(initial: &str) -> io::Result<String> {
    let path = make_temp_file_path("git-whisper-commitmsg", "txt");
    fs::write(&path, initial)?;

    let editor = env::var("EDITOR").ok().filter(|s| !s.trim().is_empty());
    let (cmd, args) = match editor {
        Some(e) => split_cmd_and_args_owned(&e),
        None => default_editor(),
    };

    let status = Command::new(cmd).args(args).arg(&path).status()?;
    if !status.success() {
        let _ = fs::remove_file(&path);
        return Err(io::Error::other(format!(
            "Editor exited with status: {status}"
        )));
    }

    let edited = fs::read_to_string(&path)?;
    let _ = fs::remove_file(&path);

    Ok(edited)
}

fn split_cmd_and_args_owned(s: &str) -> (String, Vec<String>) {
    let mut it = s.split_whitespace();
    let cmd = it.next().unwrap_or("vi").to_string();
    let args = it.map(|x| x.to_string()).collect();
    (cmd, args)
}

fn default_editor() -> (String, Vec<String>) {
    if cfg!(windows) {
        ("notepad".to_string(), vec![])
    } else {
        ("vi".to_string(), vec![])
    }
}

fn make_temp_file_path(prefix: &str, ext: &str) -> PathBuf {
    let mut p = env::temp_dir();
    let pid = std::process::id();

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    p.push(format!("{prefix}-{pid}-{nanos}.{ext}"));
    p
}
