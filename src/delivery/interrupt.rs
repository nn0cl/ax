//! Ctrl+C handler: immediate message + set core cancel flag.

use crate::core::interrupt as core_interrupt;
use std::fs::OpenOptions;
use std::io::Write;

/// Install SIGINT/Ctrl+C handler. Prints immediately, then cooperative shutdown.
pub fn install() {
    let result = ctrlc::set_handler(|| {
        core_interrupt::request_cancel();
        emit_interrupt_message();
    });
    if let Err(e) = result {
        eprintln!("warning: could not install Ctrl+C handler: {e}");
    }
}

fn emit_interrupt_message() {
    let msg = "\n\n*** interrupted: 終了処理中… / shutting down — please wait ***\n\
               (完成済み unit は plan に保存。続きは --resume)\n\n";
    if let Ok(mut tty) = OpenOptions::new().write(true).open("/dev/tty") {
        let _ = tty.write_all(msg.as_bytes());
        let _ = tty.flush();
    }
    let _ = std::io::stderr().write_all(msg.as_bytes());
    let _ = std::io::stderr().flush();
}
