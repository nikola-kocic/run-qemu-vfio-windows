mod undoable_command;

use crate::undoable_command::{new_cmd, Executor, UndoableCommand};

fn run_app() -> Result<(), std::process::ExitStatus> {
    let mut e = Executor::new();

    e.run(UndoableCommand::new(
        new_cmd("/bin/touch", &["/tmp/file1.txt"]),
        new_cmd("/bin/rm", &["/tmp/file1.txt"]),
    ))?;
    e.run(UndoableCommand::new(
        new_cmd("/bin/touch", &["/tmp/file2.txt"]),
        new_cmd("/bin/rm", &["/tmp/file2.txt"]),
    ))?;
    e.run(UndoableCommand::new(
        new_cmd("/bin/touch", &["/nntmp/file3.txt"]),
        new_cmd("/bin/rm", &["/tmp/file3.txt"]),
    ))?;

    Ok(())
}

fn main() {
    ::std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(status) => {
            eprintln!("Command exited with: {}", status);
            1
        }
    });
}
