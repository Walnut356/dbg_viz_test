#![allow(clippy::unused_io_amount)]

// use std::{io::{self, pipe, BufRead, BufReader, Read, Write}, process::{Command, Stdio}, thread};

// fn main() {
//     let (child_read, mut child_write) = pipe().unwrap();
//     let (mut parent_read, parent_write) = pipe().unwrap();

//     let mut child = Command::new("lldb")
//         .arg(r"C:\Users\ant_b\Documents\Coding Projects\dbg_viz_test\target\debug\sample.exe")
//         .stdin(Stdio::piped())
//         .stdout(Stdio::piped())
//         .spawn()
//         .unwrap();

//     // Create threads for reading stdout and stderr
//     let stdout_thread = thread::spawn(move || {
//         let stdout = child.stdout.as_mut().expect("Failed to capture stdout");
//         let stdout_reader = BufReader::new(stdout);
//         for line in stdout_reader.lines() {
//             if let Ok(line) = line {
//                 println!("{}", line);
//             }
//         }
//     });

//     // Wait for both threads to finish
//     stdout_thread.join().expect("stdout thread panicked");

//     // let mut line = String::new();

//     // parent_read.read_to_string(&mut line).unwrap();

//     // println!("{line}");

//     // while child.try_wait().is_ok_and(|x| x.is_none()) {

//     //     if let Some(stdout) = child.stdout.as_mut() {
//     //         let x = stdout.read_to_string(&mut line).unwrap();
//     //         println!("read: {line}");
//     //     }
//     //     println!("read: {line}");
//     // }
// }

use std::{process::Stdio, thread, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::Command;

#[derive(Debug, Clone, Copy)]
enum State {
    Launch,
    Running,
    Breakpoint,
    VarCheck(u32),
}

/// Pairs of values for testing. The first value in the pair is the command to make lldb print the
/// variable, the second is the output to check against. The output may span multiple lines.
const VARS: &[(&str, &str)] = &[("v u8_v", "(unsigned char) ")];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("lldb");

    cmd.arg(r"C:\Users\ant_b\Documents\Coding Projects\dbg_viz_test\target\debug\sample.exe");

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

    let mut running = false;
    let mut stopped = false;
    let mut vars = false;
    let mut var_count = 0;
    let mut line = String::new();
    loop {
        reader.read_line(&mut line).await?;
        print!("{line}");

        match state {
            State::Launch => {
                // ~indicates that lldb has parsed the debug info and is ready to accept commands
                if line.starts_with("Current executable") {
                    // prevents noise that makes it much harder to 
                    stdin
                        .write("setting set stop-line-count-before 0\n".as_bytes())
                        .await?;
                    stdin
                        .write("setting set stop-line-count-after 0\n".as_bytes())
                        .await?;
                    stdin.write("b main.rs:6\n".as_bytes()).await?;
                }
            },
            State::Running => todo!(),
            State::Breakpoint => todo!(),
            State::VarCheck(_) => todo!(),
        }

        if line.starts_with("Breakpoint") {
            stdin.write("run\n".as_bytes()).await?;
            running = true;
            thread::sleep(Duration::from_secs(1));
        }
        if line.contains("Process") && line.contains("stopped") {
            stopped = true;
        }
        if stopped && running && line.starts_with("    f") {
            reader.read_line(&mut line);
            vars = true;
            stdin.write("v u8_v\n".as_bytes()).await?;
            // line.clear();
            // reader.read_line(&mut line);
            // assert_eq!(line, "(unsigned char) u8_v = 0");
        }

        if vars {
            match var_count {
                0 => {
                    if line.contains("(lldb) v u8_v") {
                        line.clear();
                        reader.read_line(&mut line).await?;
                        assert_eq!(line, "(unsigned char) u8_v = '\\0'\n");
                        var_count += 1;
                    }
                }
                1 => if line.contains("(lldb) v ") {},
                _ => {}
            }
        }
        line.clear();
    }

    Ok(())
}
