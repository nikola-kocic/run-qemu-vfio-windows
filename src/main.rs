mod undoable_command;

use crate::undoable_command::UndoableCommand;
use std::process::Command;

fn run_app() -> Result<(), std::process::ExitStatus> {
    UndoableCommand::new(
        Command::new("/bin/touch").arg("/tmp/file.txt"),
        Command::new("/bin/rm").arg("/tmp/file.txt"),
    )
    .run()?;

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
