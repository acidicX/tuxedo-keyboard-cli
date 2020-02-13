#![recursion_limit = "1024"]

extern crate structopt;
use structopt::StructOpt;

#[macro_use]
extern crate error_chain;
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}
use errors::*;

error_chain! {
    errors {
        MissingSysFs(t: String) {
            description("invalid toolchain name")
            display("invalid toolchain name: '{}'", t)
        }
    }
}

use std::process::Command;

extern crate nix;
use nix::unistd::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "tuxedo-keyboard-cli", about = "A CLI for interfacing with the tuxedo keyboard DKMS module")]
struct Cli {
    /// color as RGB string
    color: String,

    /// brightness (0 - 100)
    #[structopt(long = "brightness", short = "b", default_value = "75")]
    brightness: u8,

    /// backlight modes (0 - 7)
    #[structopt(long = "mode", short = "m", default_value = "0")]
    mode: u8,
}

fn preflight_check() -> Result<()> {
    let check_sysfs = Command::new("ls")
        .arg("/sys/devices/platform/tuxedo_keyboard")
        .output().unwrap_or_else(|e| {
        panic!("failed to execute process: {}", e);
    });

    if check_sysfs.status.success() {
        println!("sysfs exists");
    } else {
        let s = String::from_utf8_lossy(&check_sysfs.stderr);
        //println!("sysfs missing or ls failed and stderr was:\n{}", s);
        bail!(ErrorKind::MissingSysFs("sysfs missing or ls failed".to_string()));
    }

    let check_module = Command::new("sh")
        .arg("-c")
        .arg("lsmod | grep tuxedo_keybard")
        .output().unwrap_or_else(|e| {
        panic!("failed to execute process: {}", e);
    });

    if check_module.status.success() {
        println!("kernel module is loaded");
    } else {
        let s = String::from_utf8_lossy(&check_module.stderr);
        //println!("kernel module not loaded or lsmod failed and stderr was:\n{}", s);
        bail!(ErrorKind::MissingSysFs("kernel module not loaded or lsmod failed".to_string()));
    }

    Ok(())
}

fn cli_sanity_check() {
    let cli = Cli::from_args();
    if cli.brightness > 100 {
        panic!("failed to execute process");
    }
    if cli.mode > 7 {
        panic!("failed to execute process");
    }
}

fn exec_sh(echo: String) {
    let uid = getuid();
    if uid.is_root() {
        println!("{}", "rootin tootin");
        let executed_shell = Command::new("sh")
            .arg("-c")
            .arg(&echo)
            .output().unwrap_or_else(|e| {
            panic!("failed to execute process: {}", e);
        });

        if executed_shell.status.success() {
            println!("{} {}", "shell exec succeeded!", &echo);
        } else {
            let s = String::from_utf8_lossy(&executed_shell.stderr);
            println!("shell exec failed and stderr was:\n{}", s);
        }
    } else {
        let executed_shell = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(&echo)
            .output().unwrap_or_else(|e| {
            panic!("failed to execute process: {}", e);
        });

        if executed_shell.status.success() {
            println!("{} {}", "shell exec succeeded!", &echo);
        } else {
            let s = String::from_utf8_lossy(&executed_shell.stderr);
            println!("shell exec failed and stderr was:\n{}", s);
        }
    }
}

#[allow(dead_code)]
fn main() {
    if let Err(ref e) = run() {
        use error_chain::ChainedError;
        use std::io::Write; // trait which holds `display_chain`
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e.display_chain()).expect(errmsg);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    preflight_check().unwrap_err();
    cli_sanity_check();
    let cli = Cli::from_args();

    let set_brightness_str = format!("{}{}{}", "echo ", cli.brightness, " > /sys/devices/platform/tuxedo_keyboard/brightness");
    exec_sh(set_brightness_str);

    let set_mode_str = format!("{}{}{}", "echo ", cli.mode, " > /sys/devices/platform/tuxedo_keyboard/mode");
    exec_sh(set_mode_str);

    Ok(())
}
