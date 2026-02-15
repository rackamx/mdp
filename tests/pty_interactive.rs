use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_interactive_quit_in_pty() {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("failed to open pty");

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_mdp"));
    let readme = format!("{}/README.md", env!("CARGO_MANIFEST_DIR"));
    cmd.arg(readme);
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .expect("failed to spawn mdp in pty");
    drop(pair.slave);

    let mut writer = pair.master.take_writer().expect("failed to get pty writer");
    thread::sleep(Duration::from_millis(150));
    writer.write_all(b"q").expect("failed to send quit key");
    writer.flush().expect("failed to flush quit key");

    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match child.try_wait().expect("wait failed") {
            Some(status) => {
                assert!(status.success(), "expected clean exit, got {status:?}");
                break;
            }
            None if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(50));
            }
            None => {
                child.kill().ok();
                panic!("mdp did not exit after sending quit key");
            }
        }
    }
}
