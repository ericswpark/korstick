#![windows_subsystem = "windows"]

extern crate ctrlc;
extern crate winapi;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use winapi::shared::minwindef::{DWORD, LPARAM};
use winapi::um::winuser::{
    GetForegroundWindow, GetKeyboardLayout, GetWindowThreadProcessId, LoadKeyboardLayoutW, SendMessageW, KLF_ACTIVATE, KLF_SETFORPROCESS,
    KLF_SUBSTITUTE_OK, WM_INPUTLANGCHANGEREQUEST,
};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
};

use trayicon::{MenuBuilder, TrayIcon, TrayIconBuilder};

const KOREAN_IME_LAYOUT_ID: &str = "00000412";

#[derive(Clone, Eq, PartialEq, Debug)]
enum UserEvents {
    RightClickTrayIcon,
    LeftClickTrayIcon,
    Exit,
}

fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn main() {
    let event_loop = EventLoop::<UserEvents>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();
    let icon = include_bytes!("../images/ko.ico");

    let tray_icon = TrayIconBuilder::new()
        .sender(move |e: &UserEvents| {
            let _ = proxy.send_event(e.clone());
        })
        .icon_from_buffer(icon)
        .tooltip("korstick")
        .on_click(UserEvents::LeftClickTrayIcon)
        .on_right_click(UserEvents::RightClickTrayIcon)
        .menu(MenuBuilder::new().item("E&xit", UserEvents::Exit))
        .build()
        .unwrap();

    static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

    let switcher_thread = thread::spawn(|| {
        while !EXIT_REQUESTED.load(Ordering::Relaxed) {
            switch_to_korean_layout();
            thread::sleep(Duration::from_secs(1));
        }
    });

    let mut app = MainApp { tray_icon };
    event_loop.run_app(&mut app).unwrap();

    EXIT_REQUESTED.store(true, Ordering::Relaxed);
    switcher_thread.join().unwrap();
}

fn switch_to_korean_layout() {
    unsafe {
        let hwnd = GetForegroundWindow();
        if !hwnd.is_null() {
            let new_layout: *mut winapi::shared::minwindef::HKL__ = LoadKeyboardLayoutW(
                to_wide_string(KOREAN_IME_LAYOUT_ID).as_ptr(),
                KLF_ACTIVATE | KLF_SUBSTITUTE_OK | KLF_SETFORPROCESS,
            );

            if new_layout.is_null() {
                eprintln!("Failed to load the Korean IME layout.");
            } else {
                if is_window_layout_korean(hwnd) {
                    println!("The current layout is already Korean IME.");
                    return;
                }

                SendMessageW(hwnd, WM_INPUTLANGCHANGEREQUEST, 0, new_layout as LPARAM);
                println!("Switched to the Korean IME layout.");
            }
        } else {
            eprintln!("Unable to fetch the current window!");
        }
    }
}

fn is_window_layout_korean(hwnd: winapi::shared::windef::HWND) -> bool {
    unsafe {
        let thread_id = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        let layout = GetKeyboardLayout(thread_id);
        let layout_id = layout as u32 & 0xFFFF;
        
        if layout_id == 0x0412 {
            return true;
        } else {
            return false;
        }
    }
}

struct MainApp {
    tray_icon: TrayIcon<UserEvents>,
}

impl ApplicationHandler<UserEvents> for MainApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: WindowEvent,
    ) {
    }

    // Application specific events
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvents) {
        match event {
            UserEvents::Exit => event_loop.exit(),
            UserEvents::RightClickTrayIcon => {
                self.tray_icon.show_menu().unwrap();
            }
            UserEvents::LeftClickTrayIcon => {
                self.tray_icon.show_menu().unwrap();
            }
        }
    }
}
