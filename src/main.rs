extern crate ctrlc;
extern crate winapi;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use winapi::shared::minwindef::LPARAM;
use winapi::um::winuser::{
    GetForegroundWindow, LoadKeyboardLayoutW, SendMessageW, KLF_ACTIVATE, KLF_SETFORPROCESS,
    KLF_SUBSTITUTE_OK, WM_INPUTLANGCHANGEREQUEST,
};

const KOREAN_IME_LAYOUT_ID: &str = "00000412";

fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        unsafe {
            let hwnd = GetForegroundWindow();
            if !hwnd.is_null() {
                let new_layout = LoadKeyboardLayoutW(
                    to_wide_string(KOREAN_IME_LAYOUT_ID).as_ptr(),
                    KLF_ACTIVATE | KLF_SUBSTITUTE_OK | KLF_SETFORPROCESS,
                );

                if new_layout.is_null() {
                    eprintln!("Failed to load the Korean IME layout.");
                } else {
                    SendMessageW(hwnd, WM_INPUTLANGCHANGEREQUEST, 0, new_layout as LPARAM);
                    println!("Switched to the Korean IME layout.");
                }
            } else {
                eprintln!("Unable to fetch the current window!");
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}
