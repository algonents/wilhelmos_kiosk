//! Journald-friendly logging.
//!
//! Under the WilhelmOS kiosk session, stdout/stderr are wired to journald
//! by the systemd unit. Lines prefixed with `<N>` (sd-daemon convention)
//! carry an explicit journal priority. These free functions are the whole
//! logging API — no `log` crate, no globals to configure, nothing to
//! initialize (dependency policy: `docs/DESIGN.md` §2).

/// Log at journald priority 6 (informational).
pub fn info(msg: &str) {
    eprintln!("<6>{msg}");
}

/// Log at journald priority 4 (warning).
pub fn warn(msg: &str) {
    eprintln!("<4>{msg}");
}

/// Log at journald priority 3 (error).
pub fn error(msg: &str) {
    eprintln!("<3>{msg}");
}

/// Install a panic hook that logs the panic message and location at
/// journald priority 2 (critical) before the process dies into the systemd
/// unit's `Restart=on-failure`. Installed by `Kiosk::run`; chained on top
/// of the default hook.
#[allow(dead_code)] // installed by the frame loop (`Kiosk::run`), still a stub
pub(crate) fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-string panic payload".to_string());
        eprintln!("<2>panic at {location}: {payload}");
        default_hook(info);
    }));
}
