#![recursion_limit = "1024"]

use nix::unistd::*;
use std::process::Command;
use structopt::StructOpt;

#[macro_use]
extern crate error_chain;
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}

error_chain! {
    errors {
        MissingSysFs(t: String) {
            description("sysfs missing or ls failed")
            display("sysfs missing or ls failed")
        }
        MissingKernelModule(t: String) {
            description("kernel module not loaded or lsmod failed")
            display("kernel module not loaded or lsmod failed")
        }
        WrongCliOptionRange(t: String) {
            description("a cli option has been specified with the wrong range")
            display("the cli option has been specified with the wrong range - outside '{}'", t)
        }
        WrongCliColor(t: String) {
            description("the color param has been wrongly specified")
            display("the color param has been wrongly specified, must be an RGB hex string like 'FF0000' - input: '{}'", t)
        }
        ShellExecFailed(t: String) {
            description("shell exec has failed")
            display("shell exec has failed with the following paramter: '{}'", t)
        }
    }
}

fn validate_mode(mode: &str) -> Result<u8> {
    let int_mode = mode.parse::<u8>().unwrap();
    if int_mode > 7 {
        bail!(ErrorKind::WrongCliOptionRange("0 - 7".to_string()));
    }

    Ok(int_mode)
}

fn validate_state(state: &str) -> Result<u8> {
    let parsed_state = state.parse::<u8>();
    let int_state = match parsed_state {
        Ok(int_state) => int_state,
        Err(_error) => bail!(ErrorKind::WrongCliOptionRange("0 - 1".to_string())),
    };

    if int_state > 1 {
        bail!(ErrorKind::WrongCliOptionRange("0 - 1".to_string()));
    }

    Ok(int_state)
}

fn validate_color(color: &str) -> Result<String> {
    if color.len() != 6 {
        bail!(ErrorKind::WrongCliColor(color.to_string()));
    }

    let color_result = u32::from_str_radix(color, 16).map(|o| o as u32);

    match color_result {
        Ok(color) => color,
        Err(_error) => bail!(ErrorKind::WrongCliColor(color.to_string())),
    };

    Ok(color.to_string())
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "tuxedo-keyboard-cli",
    about = "A CLI for interfacing with the tuxedo keyboard DKMS module"
)]
struct Cli {
    /// color as RGB string
    #[structopt(parse(try_from_str = validate_color))]
    color: String,

    /// brightness (0 - 255)
    #[structopt(long = "brightness", short = "b", default_value = "255")]
    brightness: u8,

    /// backlight modes (0 - 7)
    #[structopt(long = "mode", short = "m", default_value = "0", parse(try_from_str = validate_mode))]
    mode: u8,

    /// state (0 or 1)
    #[structopt(long = "state", short = "s", default_value = "1", parse(try_from_str = validate_state))]
    state: u8,
}

fn preflight_check() -> Result<()> {
    let check_sysfs = Command::new("ls")
        .arg("/sys/devices/platform/tuxedo_keyboard")
        .output()
        .unwrap();

    if !check_sysfs.status.success() {
        bail!(ErrorKind::MissingSysFs("".to_string()));
    }

    let check_module = Command::new("sh")
        .arg("-c")
        .arg("lsmod | grep tuxedo_keyboard")
        .output()
        .unwrap();

    if !check_module.status.success() {
        bail!(ErrorKind::MissingKernelModule("".to_string()));
    }

    Ok(())
}

fn exec_sh(echo: String) -> Result<()> {
    let uid = getuid();
    if uid.is_root() {
        let executed_shell = Command::new("sh").arg("-c").arg(&echo).output().unwrap();

        if !executed_shell.status.success() {
            bail!(ErrorKind::ShellExecFailed(echo));
        }
    } else {
        let executed_shell = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(&echo)
            .output()
            .unwrap();

        if !executed_shell.status.success() {
            bail!(ErrorKind::ShellExecFailed(echo));
        }
    }

    Ok(())
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
    preflight_check()?;
    let cli = Cli::from_args();

    let set_brightness_str = format!(
        "{}{}{}",
        "echo ", cli.brightness, " > /sys/devices/platform/tuxedo_keyboard/brightness"
    );
    exec_sh(set_brightness_str)?;

    let set_mode_str = format!(
        "{}{}{}",
        "echo ", cli.mode, " > /sys/devices/platform/tuxedo_keyboard/mode"
    );
    exec_sh(set_mode_str)?;

    let set_color_left_str = format!(
        "{}{}{}",
        "echo 0x", cli.color, " > /sys/devices/platform/tuxedo_keyboard/color_left"
    );
    exec_sh(set_color_left_str)?;

    let set_color_center_str = format!(
        "{}{}{}",
        "echo 0x", cli.color, " > /sys/devices/platform/tuxedo_keyboard/color_center"
    );
    exec_sh(set_color_center_str)?;

    let set_color_right_str = format!(
        "{}{}{}",
        "echo 0x", cli.color, " > /sys/devices/platform/tuxedo_keyboard/color_right"
    );
    exec_sh(set_color_right_str)?;

    let set_color_extra_str = format!(
        "{}{}{}",
        "echo 0x", cli.color, " > /sys/devices/platform/tuxedo_keyboard/color_extra"
    );
    exec_sh(set_color_extra_str)?;

    let set_state_str = format!(
        "{}{}{}",
        "echo ", cli.state, " > /sys/devices/platform/tuxedo_keyboard/state"
    );
    exec_sh(set_state_str)?;

    Ok(())
}
