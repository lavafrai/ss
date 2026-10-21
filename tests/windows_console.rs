#![cfg(windows)]

#[path = "../src/ssh.rs"]
mod ssh;

use std::{
    fs,
    io::{Read, Write},
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{self, Command, Stdio},
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
const EXPECTED_STATUS: i32 = 0x1234;
static RECEIVED_EVENTS: AtomicU32 = AtomicU32::new(0);

#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetConsoleCtrlHandler(
        handler: Option<unsafe extern "system" fn(u32) -> i32>,
        add: i32,
    ) -> i32;
    fn GenerateConsoleCtrlEvent(event: u32, process_group: u32) -> i32;
}

// The worker gets a separate console so generated events cannot interrupt cargo
// or the user's shell. Its child is this test executable copied as ssh.exe.
#[test]
fn ssh_keeps_control_events_and_parent_waits() {
    let fixture = create_fixture();
    let executable = std::env::current_exe().unwrap();
    fs::copy(&executable, fixture.join("ssh.exe")).unwrap();

    let mut worker = Command::new(executable)
        .args([
            "--exact",
            "console_worker",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("SS_CONSOLE_TEST_DIR", &fixture)
        .env("PATH", &fixture)
        .stdin(Stdio::piped())
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()
        .unwrap();
    worker
        .stdin
        .take()
        .unwrap()
        .write_all(b"input passed directly to ssh\n")
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(15);
    let status = loop {
        if let Some(status) = worker.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = worker.kill();
            let _ = worker.wait();
            panic!("console worker timed out; fixture: {}", fixture.display());
        }
        thread::sleep(Duration::from_millis(10));
    };

    let result = fs::read_to_string(fixture.join("result.txt")).unwrap_or_default();
    let _ = fs::remove_dir_all(fixture);
    assert!(status.success(), "parent exited before SSH: {status}");
    assert_eq!(result, "ctrl-c,ctrl-break,status=4660");
}

#[test]
fn console_worker() {
    let Some(fixture) = std::env::var_os("SS_CONSOLE_TEST_DIR") else {
        return;
    };

    let code = ssh::connect("fake_ssh_probe").unwrap();
    assert_eq!(
        code, EXPECTED_STATUS,
        "SSH's full Windows exit code was lost"
    );
    fs::write(
        PathBuf::from(fixture).join("result.txt"),
        "ctrl-c,ctrl-break,status=4660",
    )
    .unwrap();
}

#[test]
fn fake_ssh_probe() {
    if std::env::current_exe().unwrap().file_name().unwrap() != "ssh.exe" {
        return;
    }

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    assert_eq!(input, "input passed directly to ssh\n");

    // A process-local parent handler must not make SSH inherit "ignore Ctrl+C".
    // SAFETY: the callback has the required ABI and remains valid until exit.
    assert_ne!(unsafe { SetConsoleCtrlHandler(Some(record_event), 1) }, 0);
    for event in [0, 1] {
        // SAFETY: broadcast only inside the isolated worker console.
        assert_ne!(unsafe { GenerateConsoleCtrlEvent(event, 0) }, 0);
        let deadline = Instant::now() + Duration::from_secs(3);
        while RECEIVED_EVENTS.load(Ordering::SeqCst) & (1 << event) == 0 {
            assert!(
                Instant::now() < deadline,
                "SSH did not receive console event {event}"
            );
            thread::sleep(Duration::from_millis(10));
        }
        // Give the parent time to dispatch its independently delivered event.
        thread::sleep(Duration::from_millis(100));
    }

    process::exit(EXPECTED_STATUS);
}

unsafe extern "system" fn record_event(event: u32) -> i32 {
    if event <= 1 {
        RECEIVED_EVENTS.fetch_or(1 << event, Ordering::SeqCst);
        1
    } else {
        0
    }
}

fn create_fixture() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("windows-console-{}-{suffix}", process::id()));
    fs::create_dir_all(&fixture).unwrap();
    fixture
}
