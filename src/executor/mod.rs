use anyhow::Result;
use std::process::{Command, Stdio};
use std::time::Instant;

pub struct ExecResult {
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    pub success: bool,
}

pub fn execute(command: &str, cwd: Option<&str>, timeout_secs: u64) -> Result<ExecResult> {
    // NOTE: timeout is accepted but not enforced yet (kept for a future improvement).
    let _ = timeout_secs;
    let start = Instant::now();

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", command]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", command]);
        c
    };

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());

    let output = cmd.output()?;
    let duration = start.elapsed();

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code();
    let success = output.status.success();

    Ok(ExecResult {
        command: command.to_string(),
        stdout,
        stderr,
        exit_code,
        duration_ms: duration.as_millis(),
        success,
    })
}
