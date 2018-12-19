use std::process::Command;

pub struct UndoableCommand<'a> {
    cmd: &'a mut Command,
    undo: &'a mut Command,
    cmd_executed: bool,
}

fn verbose_execute_cmd(cmd: &mut Command) -> Result<(), std::process::ExitStatus> {
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
        Ok(())
    }
}

impl<'a> UndoableCommand<'a> {
    pub fn new(cmd: &'a mut Command, undo: &'a mut Command) -> UndoableCommand<'a> {
        UndoableCommand {
            cmd,
            undo,
            cmd_executed: false,
        }
    }

    pub fn run(&mut self) -> Result<(), std::process::ExitStatus> {
        verbose_execute_cmd(&mut self.cmd).map(|v| {
            self.cmd_executed = true;
            v
        })
    }
}

impl<'a> Drop for UndoableCommand<'a> {
    fn drop(&mut self) {
        if self.cmd_executed {
            verbose_execute_cmd(&mut self.undo).unwrap_or_else(|e| {
                eprintln!("Command exited with: {}", e);
            });
        }
    }
}
