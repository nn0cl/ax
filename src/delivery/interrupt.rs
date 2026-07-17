//! Ctrl+C handler: message + cooperative cancel; 2nd press force-exits.

use crate::core::interrupt as core_interrupt;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

static MSG_PRINTED: AtomicBool = AtomicBool::new(false);

/// Install SIGINT/Ctrl+C handler.
pub fn install() {
    let result = ctrlc::set_handler(|| {
        let first = core_interrupt::on_ctrl_c();
        if first && !MSG_PRINTED.swap(true, Ordering::SeqCst) {
            emit_interrupt_message();
        }
        // 2nd press never returns from on_ctrl_c() (libc::_exit).
    });
    if let Err(e) = result {
        eprintln!("warning: could not install Ctrl+C handler: {e}");
    }
}

fn emit_interrupt_message() {
    let msg = "\n\n*** interrupted: 終了処理中… / shutting down — please wait ***\n\
               (完成済み unit は plan に保存。続きは --resume)\n\
               >>> もう一度 Ctrl+C で強制終了 / second Ctrl+C = force exit <<<\n\n";
    // Best-effort only; may not be async-signal-safe. Force path uses libc only.
    if let Ok(mut tty) = OpenOptions::new().write(true).open("/dev/tty") {
        let _ = tty.write_all(msg.as_bytes());
        let _ = tty.flush();
    }
    let _ = std::io::stderr().write_all(msg.as_bytes());
    let _ = std::io::stderr().flush();
}
