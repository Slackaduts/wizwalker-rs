use std::{ffi::OsString, os::windows::ffi::OsStringExt};
use std::fmt::Debug;

use winapi::um::processthreadsapi::GetExitCodeProcess;
use winapi::um::sysinfoapi::GetSystemDirectoryW;
use winapi::um::winnt::HANDLE;
use winapi::shared::minwindef::DWORD;
use anyhow::{Result, anyhow};

use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE,
};

use std::ffi::CStr;
use std::mem::{size_of, zeroed};



pub fn check_if_process_running(handle: HANDLE) -> Result<bool> {
    let mut exit_code: DWORD = 0;

    unsafe {
        GetExitCodeProcess(handle, &mut exit_code as *mut DWORD)
    };

    match exit_code {
        259 => Ok(true),
        0 => Ok(false),
        _ => Err(anyhow!("Unknown process exit code: {exit_code}")),
    }
}

//     return Path(buffer.value)
pub fn get_system_directory(max_size: usize) -> String {
    let mut buffer = vec![0u16; max_size];

    let len = unsafe {
        GetSystemDirectoryW(buffer.as_mut_ptr(), max_size as u32)
    };

    buffer.truncate(len as usize);

    let os_str = OsString::from_wide(&buffer);
    os_str.to_string_lossy().into_owned()
}


pub struct Orient {
    pitch: f64,
    roll: f64,
    yaw: f64
}

impl Debug for Orient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Orient (Pitch: {}, roll: {}, yaw: {})>", 
            &self.pitch, &self.roll, &self.yaw)
    }
}


/// Analogue for pymem.module_from_name()
pub fn module_from_name(process_id: u32, module_name: &str) -> Option<MODULEENTRY32> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, process_id);
        if snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
            return None;
        }

        let mut module_entry: MODULEENTRY32 = zeroed();
        module_entry.dwSize = size_of::<MODULEENTRY32>() as u32;

        if Module32First(snapshot, &mut module_entry) == 0 {
            return None;
        }

        loop {
            let module_entry_name = CStr::from_ptr(module_entry.szModule.as_ptr());
            if let Ok(name) = module_entry_name.to_str() {
                if name.eq_ignore_ascii_case(module_name) {
                    return Some(module_entry);
                }
            }

            if Module32Next(snapshot, &mut module_entry) == 0 {
                break;
            }
        }
    }
    None
}
