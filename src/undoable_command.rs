use std::process::Command;

pub struct UndoableCommand {
    run_cmd: Command,
    undo_cmd: Command,
    cmd_executed: bool,
    undo_executed: bool,
}

fn verbose_execute_cmd(cmd: &mut Command) -> Result<String, std::process::ExitStatus> {
    println!("running: {:?}", cmd);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        println!("  stdout: {}", stdout);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("  stderr: {}", stderr);
    }
    if !output.status.success() {
        Err(output.status)
    } else {
        Ok(stdout.into())
    }
}

pub fn new_cmd<S, SI, I>(cmd_bin: S, cmd_args: I) -> Command
where
    S: AsRef<std::ffi::OsStr>,
    SI: AsRef<std::ffi::OsStr>,
    I: IntoIterator<Item = SI>,
{
    let mut cmd = Command::new(cmd_bin);
    cmd.args(cmd_args);
    cmd
}

impl UndoableCommand {
    pub fn new(run_cmd: Command, undo_cmd: Command) -> UndoableCommand {
        UndoableCommand {
            run_cmd,
            undo_cmd,
            cmd_executed: false,
            undo_executed: false,
        }
    }

    pub fn run(&mut self) -> Result<String, std::process::ExitStatus> {
        verbose_execute_cmd(&mut self.run_cmd).map(|v| {
            self.cmd_executed = true;
            v
        })
    }

    pub fn undo(&mut self) -> Result<String, std::process::ExitStatus> {
        println!("Called undo for {:?}", self.undo_cmd);
        if self.undo_executed {
            eprintln!(
                "!!! Undo command already executed, skipping: {:?}",
                self.undo_cmd
            );
            Ok(String::new())
        } else if self.cmd_executed {
            verbose_execute_cmd(&mut self.undo_cmd).map(|v| {
                self.undo_executed = true;
                v
            })
        } else {
            Ok(String::new())
        }
    }
}

pub struct Executor {
    cmds_executed: Vec<UndoableCommand>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            cmds_executed: Vec::new(),
        }
    }

    pub fn run(&mut self, mut cmd: UndoableCommand) -> Result<String, std::process::ExitStatus> {
        cmd.run().map(|v| {
            self.cmds_executed.push(cmd);
            v
        })
    }

    pub fn undo_all(&mut self) {
        // last executed command should be first undoed
        while let Some(mut cmd) = self.cmds_executed.pop() {
            match cmd.undo() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("!!! Undo command exited with: {}", e);
                }
            };
        }
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        self.undo_all();
    }
}
