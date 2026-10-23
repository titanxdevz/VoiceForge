//! Windows Multimedia Class Scheduler Service (MMCSS) registration
//! Ensures the real-time audio thread gets maximum priority and isn't preempted.

#[cfg(windows)]
pub fn set_audio_thread_priority() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let task_name: Vec<u16> = OsStr::new("Pro Audio").encode_wide().chain(Some(0)).collect();
    let mut task_index: u32 = 0;

    unsafe {
        // Load Avrt.dll dynamically or call AvSetMmThreadCharacteristicsW
        type AvSetMmThreadCharacteristicsFn = unsafe extern "system" fn(*const u16, *mut u32) -> *mut std::ffi::c_void;
        
        let avrt = windows_sys::Win32::System::LibraryLoader::LoadLibraryA(c"Avrt.dll".as_ptr() as *const u8);
        if !avrt.is_null() {
            let proc = windows_sys::Win32::System::LibraryLoader::GetProcAddress(avrt, c"AvSetMmThreadCharacteristicsW".as_ptr() as *const u8);
            if let Some(func) = proc {
                let av_set: AvSetMmThreadCharacteristicsFn = std::mem::transmute(func);
                let handle = av_set(task_name.as_ptr(), &mut task_index);
                return !handle.is_null();
            }
        }
    }
    false
}

#[cfg(not(windows))]
pub fn set_audio_thread_priority() -> bool {
    false
}
