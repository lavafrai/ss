use std::{
    io,
    process::{Command, Stdio},
};

#[cfg(windows)]
#[path = "ssh/windows.rs"]
mod windows;

pub fn connect(alias: &str) -> io::Result<i32> {
    let mut command = Command::new("ssh");
    command
        .args(["--", alias])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    launch(&mut command)
        .map_err(|error| io::Error::new(error.kind(), format!("failed to start ssh: {error}")))
}

#[cfg(unix)]
fn launch(command: &mut Command) -> io::Result<i32> {
    use std::os::unix::process::CommandExt;

    // Replace ss in the same foreground process group and controlling terminal.
    // Success never returns; the shell receives SSH's status and signals directly.
    Err(command.exec())
}

#[cfg(windows)]
fn launch(command: &mut Command) -> io::Result<i32> {
    // The child inherits the current console; no new window or process group.
    let _ctrl_handler = windows::ConsoleCtrlGuard::install()?;
    let status = command.status()?;
    Ok(status.code().unwrap_or(1))
}
