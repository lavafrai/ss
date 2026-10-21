use std::io;

const CTRL_C_EVENT: u32 = 0;
const CTRL_BREAK_EVENT: u32 = 1;

type ConsoleHandler = unsafe extern "system" fn(u32) -> i32;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetConsoleCtrlHandler(handler: Option<ConsoleHandler>, add: i32) -> i32;
}

pub(super) struct ConsoleCtrlGuard;

impl ConsoleCtrlGuard {
    pub(super) fn install() -> io::Result<Self> {
        // SAFETY: the callback has the Win32 signature and lives for the whole process.
        if unsafe { SetConsoleCtrlHandler(Some(wait_for_ssh), 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self)
    }
}

impl Drop for ConsoleCtrlGuard {
    fn drop(&mut self) {
        // SAFETY: remove the same callback registered by install().
        unsafe { SetConsoleCtrlHandler(Some(wait_for_ssh), 0) };
    }
}

unsafe extern "system" fn wait_for_ssh(event: u32) -> i32 {
    // Handle these events only in ss. Windows independently delivers them to SSH.
    // A callback is process-local; SetConsoleCtrlHandler(None, TRUE) would instead
    // make the child inherit an unwanted "ignore Ctrl+C" flag.
    i32::from(matches!(event, CTRL_C_EVENT | CTRL_BREAK_EVENT))
}
