use std::collections::HashMap;
use std::default;
use std::io::{Cursor, Read, Seek};
use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::fmt::Debug;

use byteorder::{LittleEndian, ReadBytesExt};
use winapi::um::processthreadsapi::GetExitCodeProcess;
use winapi::um::sysinfoapi::GetSystemDirectoryW;
use winapi::um::winnt::HANDLE;
use winapi::shared::minwindef::DWORD;
use anyhow::{Result, anyhow};

use flate2::read::ZlibDecoder;

use directories::ProjectDirs;

use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE,
};


// use std::ptr::null_mut;
// use winapi::shared::minwindef::HKEY;
// use winapi::um::winreg::{
//     RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER
// };
// use winapi::um::winnt::{LPWSTR, KEY_READ, REG_SZ, WCHAR};

use std::ffi::CStr;
use std::mem::{size_of, zeroed};


pub const DEFAULT_INSTALL: &str = r"C:\ProgramData\KingsIsle Entertainment\Wizard101";
pub const DEFAULT_STEAM_INSTALL: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Wizard101";



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


pub fn get_wiz_install(path: Option<&str>) -> Result<PathBuf> {
    let mut install_paths: Vec<&str> = vec![
        DEFAULT_INSTALL, 
        DEFAULT_STEAM_INSTALL
    ];

    if let Some(override_path) = path {
        install_paths.insert(0, override_path);
    }


    for path_str in install_paths {
        let path = PathBuf::from(path_str);

        if path.join(r"\Wizard101.exe").exists() {
            return Ok(path)
        }
    }

    return Err(anyhow!("Unable to find Wizard101 installation in default or Steam install directories."))
}


pub fn get_cache_folder() -> Option<PathBuf> {
    //Not sure what the implications of this are. I don't want to take credit for others work OR make more work for people. - Slack
    if let Some(proj_dirs) = ProjectDirs::from("", "wizwalker-rs", "Slackaduts") {
        Some(proj_dirs.cache_dir().to_path_buf())
    } else {
        None
    }
}


pub fn parse_template_id_file(file_data: Vec<u8>) -> Result<HashMap<i32, String>> {
    let file_data_str = String::from_utf8(file_data.clone())?;

    if !file_data_str.starts_with("BINd") {
        return Err(anyhow!("No BINd id string"))
    }

    let mut decoder = ZlibDecoder::new(file_data.as_slice());
    let mut data: Vec<u8> = Vec::new();

    let total_size = decoder.read_to_end(&mut data)?;

    let mut data_slice = &data[0xD..];

    let mut cursor = Cursor::new(&mut data_slice);

    cursor.seek(std::io::SeekFrom::Start(0x24))?;

    let mut out: HashMap<i32, String> = HashMap::new();

    while cursor.position() < total_size as u64 {
        let mut char_buf: [u8; 1] = [0u8];
        cursor.read_exact(&mut char_buf)?;

        let size: u8 = char_buf[0] / 2;

        let mut string_buf = Vec::with_capacity(size as usize);
        cursor.read_exact(&mut string_buf)?;

        let string = String::from_utf8(string_buf)?;

        let entry_id = cursor.read_i32::<LittleEndian>()?;

        cursor.read_exact(&mut Vec::with_capacity(0x10))?;

        out.insert(entry_id, string);
    }

    Ok(out)
}