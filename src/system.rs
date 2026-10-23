use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(
        lpMutexAttributes: *const std::ffi::c_void,
        bInitialOwner: i32,
        lpName: *const u16,
    ) -> HANDLE;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SingleInstanceError {
    AlreadyRunning,
    CreationFailed,
}

pub struct SingleInstanceGuard {
    handle: HANDLE,
}

impl SingleInstanceGuard {
    pub fn try_acquire(name: &str) -> Result<Self, SingleInstanceError> {
        let wide_name: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
        unsafe {
            let handle = CreateMutexW(std::ptr::null(), 1, wide_name.as_ptr());
            if handle.is_null() {
                return Err(SingleInstanceError::CreationFailed);
            }
            let last_error = GetLastError();
            if last_error == 183 { // ERROR_ALREADY_EXISTS
                CloseHandle(handle);
                return Err(SingleInstanceError::AlreadyRunning);
            }
            Ok(Self { handle })
        }
    }

    pub fn focus_existing_window(window_title: &str) {
        let wide_title: Vec<u16> = OsStr::new(window_title).encode_wide().chain(Some(0)).collect();
        unsafe {
            let hwnd = FindWindowW(std::ptr::null(), wide_title.as_ptr());
            if !hwnd.is_null() {
                ShowWindow(hwnd, SW_RESTORE);
                SetForegroundWindow(hwnd);
            }
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
}

pub struct AutostartManager;

impl AutostartManager {
    const REG_SUBKEY: &'static str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const APP_NAME: &'static str = "VoiceForge";

    pub fn set_autostart(enable: bool) -> Result<(), String> {
        let subkey_wide: Vec<u16> = OsStr::new(Self::REG_SUBKEY).encode_wide().chain(Some(0)).collect();
        let app_name_wide: Vec<u16> = OsStr::new(Self::APP_NAME).encode_wide().chain(Some(0)).collect();

        unsafe {
            let mut hkey: HKEY = std::ptr::null_mut();
            let res = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey_wide.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            );
            if res != 0 {
                return Err(format!("Failed to open registry key: error code {}", res));
            }

            if enable {
                if let Ok(exe_path) = std::env::current_exe() {
                    let path_str = format!("\"{}\" --minimized", exe_path.to_string_lossy());
                    let path_wide: Vec<u16> = OsStr::new(&path_str).encode_wide().chain(Some(0)).collect();
                    let byte_len = (path_wide.len() * 2) as u32;

                    let set_res = RegSetValueExW(
                        hkey,
                        app_name_wide.as_ptr(),
                        0,
                        REG_SZ,
                        path_wide.as_ptr() as *const u8,
                        byte_len,
                    );
                    RegCloseKey(hkey);
                    if set_res != 0 {
                        return Err(format!("Failed to set autostart registry value: {}", set_res));
                    }
                }
            } else {
                let del_res = RegDeleteValueW(hkey, app_name_wide.as_ptr());
                RegCloseKey(hkey);
                if del_res != 0 && del_res != 2 {
                    return Err(format!("Failed to remove autostart registry value: {}", del_res));
                }
            }
        }
        Ok(())
    }
}
