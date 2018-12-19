use std::process::{Child, Command};

pub fn verbose_execute_cmd(cmd: &mut Command) -> Result<String, String> {
    println!("running: {:?}", cmd);
    let output = cmd.output().map_err(|e| {
        eprintln!("!!! Error running command");
        e.to_string()
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        println!("  stdout: {}", stdout);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("  stderr: {}", stderr);
    }
    if output.status.success() {
        Ok(stdout)
    } else {
        eprintln!("!!! Error running command");
        Err(output.status.to_string())
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

struct UndoableCommand {
    run_cmd: Command,
    undo_cmd: Command,
}

pub struct Executor {
    cmds_executed: Vec<UndoableCommand>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            cmds_executed: Vec::new(),
        }
    }

    pub fn run(&mut self, run_cmd: Command, undo_cmd: Command) -> Result<String, String> {
        let mut cmd = UndoableCommand { run_cmd, undo_cmd };
        verbose_execute_cmd(&mut cmd.run_cmd).map(|v| {
            self.cmds_executed.push(cmd);
            v
        })
    }

    pub fn undo_all(&mut self) {
        // last executed command should be first undoed
        while let Some(mut cmd) = self.cmds_executed.pop() {
            println!("Called undo for {:?}", cmd.run_cmd);
            match verbose_execute_cmd(&mut cmd.undo_cmd) {
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
