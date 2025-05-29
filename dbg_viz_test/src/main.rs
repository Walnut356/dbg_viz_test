#![allow(clippy::unused_io_amount)]
use std::{process::Stdio, thread, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::Command;

const LLDB_LOOKUP: &str = r"C:\Users\ant_b\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\etc\lldb_lookup.py";
const LLDB_COMMANDS: &str = r"C:\Users\ant_b\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\etc\lldb_commands";

#[derive(Debug, Clone, Copy)]
enum State {
    Launch,
    Ready,
    Running,
    VarCheck(u32),
}

/// Pairs of values for testing. The first value in the pair is the command to make lldb print the
/// variable, the second is the output to check against. The output may span multiple lines.
const CASES: &[&str] = &[
    r"(unsigned char) u8_v = 0",
    r"(unsigned short) u16_v = 1",
    r"(unsigned int) u32_v = 2",
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("lldb");

    cmd.arg(r"./target\debug\sample.exe");

    // Specify that we want the command's standard output piped back to us.
    // By default, standard input/output/error will be inherited from the
    // current process (for example, this means that standard input will
    // come from the keyboard and standard output/error will go directly to
    // the terminal if this process is invoked from the command line).
    cmd.stdout(Stdio::piped());
    cmd.stdin(Stdio::piped());

    let mut child = cmd.spawn().expect("failed to spawn command");

    let stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");
    let mut stdin = child.stdin.take().unwrap();

    let mut reader = BufReader::new(stdout);

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async move {
        let status = child
            .wait()
            .await
            .expect("child process encountered an error");

        println!("child status was: {}", status);
    });

    let mut state = State::Launch;

    let mut line = String::new();
    loop {
        line.clear();
        reader.read_line(&mut line).await?;
        print!("{line}");

        match state {
            State::Launch => {
                // ~indicates that lldb has parsed the debug info and is ready to accept commands
                if line.starts_with("Current executable") {
                    // prevents noise that makes it much harder to determine what line we're at
                    // later
                    stdin
                        .write("setting set stop-line-count-before 0\n".as_bytes())
                        .await?;
                    stdin
                        .write("setting set stop-line-count-after 0\n".as_bytes())
                        .await?;
                    // load rust's formatters
                    stdin
                        .write(format!("command script import \"{LLDB_LOOKUP}\"\n").as_bytes())
                        .await?;
                    stdin
                        .write(format!("command source -s true \"{LLDB_COMMANDS}\"\n").as_bytes())
                        .await?;
                    stdin.write("b main.rs:6\n".as_bytes()).await?;

                    state = State::Ready;
                }
            }
            State::Ready => {
                if line.starts_with("Breakpoint") {
                    stdin.write("run\n".as_bytes()).await?;
                    state = State::Running;
                }
            }
            State::Running => {
                // corresponds to the line:
                // "frame #0: 0x00007ff683421158 sample.exe`sample::main at main.rs:N"
                if line.starts_with("    f") {
                    stdin.write("v\n".as_bytes()).await?;

                    // consume the line we just wrote
                    reader.read_line(&mut line).await?;
                    state = State::VarCheck(0);
                }
            }
            State::VarCheck(idx) => {
                let expected = CASES[idx as usize];
                // check against the output, which can span multiple lines.

                let mut lines_iter = expected.lines();

                // we already have the first line, so we check that first, then do loops to read
                // more lines only if necessary.
                let e = lines_iter.next().unwrap();
                assert_eq!(e.trim(), line.trim());

                for e in lines_iter {
                    line.clear();
                    reader.read_line(&mut line).await?;
                    assert_eq!(e.trim(), line.trim());
                }

                if (idx + 1) as usize >= CASES.len() {
                    println!("\n\nALL TEST CASES OKAY");
                    return Ok(());
                }

                state = State::VarCheck(idx + 1);
            }
        }
    }

    Ok(())
}
