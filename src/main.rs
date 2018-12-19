mod undoable_command;

use crate::undoable_command::{new_cmd, verbose_execute_cmd, Executor};
use std::ffi::OsStr;
use std::path::Path;

const RAID_DEV: &str = "/dev/md1";
const MRP_DEV: &str = "/dev/disk/by-partuuid/5268dd48-f50a-467c-9a68-f56ae720d21d"; // /dev/sda2
const WIN_DEV: &str = "/dev/disk/by-partuuid/d32ffb54-5a02-42ab-9701-6283c4a2da90"; // /dev/sda3
const WIN_REC_DEV: &str = "/dev/disk/by-partuuid/e663c1c0-ad38-4752-81d4-399e4c7a6d8f"; // /dev/sda4
const _DATA_DISK_DEV: &str = "/dev/disk/by-id/wwn-0x50014ee2b29ae315"; // /dev/sdb

const USB_DEV1: &str = "09:00.0";

fn run_app(root_dir: &Path) -> Result<(), String> {
    let mut e = Executor::new();
    if Path::new(RAID_DEV).exists() {
        return Err("Raid already active".to_string());
    }

    if !root_dir.is_dir() {
        return Err("Root dir does not exist".to_string());
    }

    {
        let gptloopdev = verbose_execute_cmd(&mut new_cmd("losetup", &["-f"]))?;
        e.run(
            new_cmd(
                "losetup",
                &[
                    OsStr::new(&gptloopdev),
                    root_dir.join("gpt.raw").as_os_str(),
                ],
            ),
            new_cmd("losetup", &["--detach", &gptloopdev]),
        )?;

        let efiloopdev = verbose_execute_cmd(&mut new_cmd("losetup", &["-f"]))?;
        e.run(
            new_cmd(
                "losetup",
                &[
                    OsStr::new(&efiloopdev),
                    root_dir.join("efi.raw").as_os_str(),
                ],
            ),
            new_cmd("losetup", &["--detach", &efiloopdev]),
        )?;

        let gptbakloopdev = verbose_execute_cmd(&mut new_cmd("losetup", &["-f"]))?;
        e.run(
            new_cmd(
                "losetup",
                &[
                    OsStr::new(&gptbakloopdev),
                    root_dir.join("gpt-backup.raw").as_os_str(),
                ],
            ),
            new_cmd("losetup", &["--detach", &gptbakloopdev]),
        )?;

        verbose_execute_cmd(&mut new_cmd("modprobe", &["linear"]))?;

        e.run(
            new_cmd(
                "mdadm",
                &[
                    "--build",
                    "--verbose",
                    RAID_DEV,
                    "--level=linear",
                    "--raid-devices=6",
                    &gptloopdev,
                    &efiloopdev,
                    MRP_DEV,
                    WIN_DEV,
                    WIN_REC_DEV,
                    &gptbakloopdev,
                ],
            ),
            new_cmd("mdadm", &["--manage", "--stop", RAID_DEV]),
        )?;
    }

    {
        e.run(new_cmd("xset", &["-dpms"]), new_cmd("xset", &["+dpms"]))?;
        e.run(
            new_cmd("xset", &["s", "off"]),
            new_cmd("xset", &["s", "on"]),
        )?;
    }

    e.run(
        new_cmd("vfio-unbind", &[USB_DEV1]),
        new_cmd("vfio-restore", &[USB_DEV1, "true"]),
    )?;

    verbose_execute_cmd(&mut new_cmd("qemu-system-x86_64", vec![
        "-serial", "none",
        "-parallel", "none",
        "-nodefaults",
        "-no-user-config",
        "-enable-kvm",
        "-name", "Windows",
        "-monitor", "telnet:0.0.0.0:1234,server,nowait",
        "-cpu", "host,kvm=off,hv_vapic,hv_time,hv_relaxed,hv_spinlocks=0x1fff,hv_vendor_id=sugoidesu",
        "-smp", "sockets=1,cores=3,threads=1",
        "-m", "8192",
        "-machine", "pc,accel=kvm,kernel_irqchip=on,mem-merge=off",
        "-no-hpet",
        "-global", "kvm-pit.lost_tick_policy=discard",
        "-global", "PIIX4_PM.disable_s3=1",
        "-global", "PIIX4_PM.disable_s4=1",
        "-drive", &format!("if=pflash,format=raw,readonly,file={}", root_dir.join("Windows_ovmf_x64.bin").to_string_lossy()),
        "-drive", &format!("if=pflash,format=raw,file={}", root_dir.join("Windows_ovmf_vars_x64.bin").to_string_lossy()),
        "-rtc", "base=utc,clock=host,driftfix=none",
        "-device", "ahci,id=ahci",
        "-drive", &format!("if=none,file={},cache=none,format=raw,aio=native,id=hdd_win", RAID_DEV),
        "-device", "ide-hd,bus=ahci.0,drive=hdd_win",
        "-drive", "file=/usr/share/virtio/virtio-win.iso,index=3,media=cdrom",
        "-drive", &format!("if=none,id=drive-virtio-disk0,cache=none,aio=native,format=raw,file={},index=1", root_dir.join("dummyvirtstor.raw").to_string_lossy()),
        "-object", "iothread,id=iothread0",
        "-device", "virtio-blk-pci,iothread=iothread0,drive=drive-virtio-disk0,scsi=off,id=virtio-disk0",
        "-usb",
        "-device", "usb-kbd",
        "-device", "usb-tablet,id=input-tablet",
        "-device", "qxl-vga,revision=4,id=video0,ram_size=134217728,vram_size=134217728,vgamem_mb=128"
    ]))?;

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
