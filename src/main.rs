mod undoable_command;

use crate::undoable_command::{new_cmd, Executor, UndoableCommand, verbose_execute_cmd};
use std::path::Path;
use std::ffi::OsStr;

const RAID_DEV: &str = "/dev/md1";
// const MRP_DEV: &str = "/dev/disk/by-partuuid/5268dd48-f50a-467c-9a68-f56ae720d21d";  // /dev/sda2
// const WIN_DEV: &str = "/dev/disk/by-partuuid/d32ffb54-5a02-42ab-9701-6283c4a2da90";  // /dev/sda3
// const WIN_REC_DEV: &str = "/dev/disk/by-partuuid/e663c1c0-ad38-4752-81d4-399e4c7a6d8f";  // /dev/sda4
// const DATA_DISK_DEV: &str = "/dev/disk/by-id/wwn-0x50014ee2b29ae315"; // /dev/sdb

fn run_app(root_dir: &Path) -> Result<(), String> {
    let mut e = Executor::new();
    if Path::new(RAID_DEV).exists() {
        return Err("Raid already active".to_string());
    }

    if !root_dir.is_dir() {
        return Err("Root dir does not exist".to_string());
    }

// gptloopdev=$(losetup -f)
// losetup "${gptloopdev}" gpt.raw
// efiloopdev=$(losetup -f)
// losetup "${efiloopdev}" efi.raw
// gptbakloopdev=$(losetup -f)
// losetup "${gptbakloopdev}" gpt-backup.raw
// modprobe linear
// mdadm --build --verbose "${RAID_DEV}" --level=linear --raid-devices=6 "${gptloopdev}" "${efiloopdev}" "${MRP_DEV}" "${WIN_DEV}" "${WIN_REC_DEV}" "${gptbakloopdev}"

    let gptloopdev = verbose_execute_cmd(&mut new_cmd("losetup", &["-f"]))?;
    e.run(UndoableCommand::new(
        new_cmd("losetup", &[OsStr::new(&gptloopdev), root_dir.join("gpt.raw").as_os_str()]),
        new_cmd("losetup", &["--detach", &gptloopdev])
    ))?;




    e.run(UndoableCommand::new(
        new_cmd("/bin/touch", &["/tmp/file1.txt"]),
        new_cmd("/bin/rm", &["/tmp/file1.txt"]),
    ))?;
    e.run(UndoableCommand::new(
        new_cmd("/bin/touch", &["/tmp/file2.txt"]),
        new_cmd("/bin/rm", &["/tmp/file2.txt"]),
    ))?;
    let s: String = e.run(UndoableCommand::new(
        new_cmd("/bin/touch", &["/nntmp/file3.txt"]),
        new_cmd("/bin/rm", &["/tmp/file3.txt"]),
    ))?;

    Ok(())
}

fn main() {
    let root_dir = Path::new("/mnt/f/virtual-machines/win-ssd-partition/");

    ::std::process::exit(match run_app(root_dir) {
        Ok(_) => 0,
        Err(status) => {
            eprintln!("Command exited with: {}", status);
            1
        }
    });
}
