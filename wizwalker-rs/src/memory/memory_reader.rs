use anyhow::{anyhow, Result};
use pelite::pe::exports::By;
use tokio::sync::Mutex;
use winapi::shared::minwindef::LPVOID;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use std::collections::HashMap;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::Arc;
use tokio::task;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualAllocEx, VirtualFreeEx, VirtualQueryEx, WriteProcessMemory};
use winapi::um::winnt::{HANDLE, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE};
use winapi::um::{handleapi::CloseHandle, tlhelp32::MODULEENTRY32};
use pelite::pe::{Pe, PeFile};
use std::{fs::File, io::Read, fmt::Debug};
use region::Protection;
use winapi::ctypes::c_void;
use regex::bytes::Regex;
use winapi::um::processthreadsapi::CreateRemoteThread;

use bytemuck::{bytes_of, from_bytes, Pod, Zeroable};

use crate::utils::{check_if_process_running, get_system_directory, module_from_name};


fn addr_from_by(by: By<'_, PeFile<'_>>) -> Result<u32> {
    let funcs = by.functions();

    let addr = funcs.get(0);

    match addr {
        Some(a) => Ok(a.to_owned()),
        None => {
            Err(
                anyhow!("Could not get address for function \"{:#?}\"", by.dll_name())
            )
        }
    }
}



// Define a struct for handling memory reading operations
#[derive(Debug)]
pub struct MemoryReader {
    process: Arc<Mutex<HANDLE>>,
    symbol_table: HashMap<String, HashMap<String, u32>>, // Simplified symbol table
}

impl MemoryReader {
    // Constructor for MemoryReader
    pub fn new(process: HANDLE) -> Self {
        MemoryReader {
            process: Arc::new(Mutex::new(process)),
            symbol_table: HashMap::new(),
        }
    }

    // Simulate checking if a process is running
    pub async fn is_running(&self) -> Result<bool> {
        let ptr: *mut c_void = *self.process.lock().await;
        check_if_process_running(ptr)
    }

    // Simulate running a blocking function in an executor
    pub async fn run_in_executor<F, R>(&mut self, func: F) -> Result<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        task::spawn_blocking(func).await.map_err(|e| anyhow!(e.to_string()))
    }

    pub fn get_symbols(&mut self, file_path: &str, force_reload: bool) -> Result<HashMap<String, u32>> {
        // Early return if symbols are cached and no force reload is requested
        if !force_reload {
            if let Some(dll_table) = self.symbol_table.get(file_path) {
                return Ok(dll_table.clone());
            }
        }

        // Read the PE file
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Parse the PE file
        let pe = PeFile::from_bytes(&buffer)?;
        let mut symbols: HashMap<String, u32> = HashMap::new();

        let exports = pe.exports()?;

        for export in exports.by() {
            let (name, addr) = if let Ok(dll_name) = export.dll_name() {
                (dll_name.to_str()?.to_string(), addr_from_by(export)?)
            } else {
                (format!("Ordinal {}", export.ordinal_base().to_string()), addr_from_by(export)?)
            };

            symbols.insert(name, addr);
        }


        // Store in cache
        self.symbol_table.insert(file_path.to_string(), symbols.clone());
        
        // Return reference to stored symbols
        Ok(symbols)
    }

    pub fn scan_page_return_all(&mut self, handle: HANDLE, address: usize, pattern: &str) -> Result<(usize, Vec<usize>)> {
        let mut mbi: MEMORY_BASIC_INFORMATION = unsafe {
            std::mem::zeroed()
        };

        let result = unsafe {
            VirtualQueryEx(
                handle,
                address as *const _,
                &mut mbi,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };

        if result == 0 {
            unsafe { CloseHandle(handle) };
            return Err(anyhow!("Failed to query memory region"));
        }

        let next_region = mbi.BaseAddress as usize + mbi.RegionSize;

        // Define allowed protections
        let allowed_protections = [
            Protection::READ_EXECUTE,
            Protection::READ_WRITE_EXECUTE,
            Protection::READ_WRITE,
            Protection::READ,
        ];
    
        if mbi.State != winapi::um::winnt::MEM_COMMIT || !allowed_protections.contains(&Protection::from_bits_truncate(mbi.Protect.try_into()?)) {
            return Ok((next_region, Vec::new()));
        }
        let mut buffer = vec![0u8; mbi.RegionSize];
        let mut bytes_read: usize = 0;
        let read_result = unsafe {
            ReadProcessMemory(
                handle,
                address as *const c_void,
                buffer.as_mut_ptr() as *mut c_void,
                mbi.RegionSize,
                &mut bytes_read,
            )
        };
    
        if read_result == 0 {
            unsafe { CloseHandle(handle) };
            return Err(anyhow!("Failed to read process memory"));
        }
    
        // Search for the pattern using regex
        let regex = Regex::new(pattern)?;
        let mut found: Vec<usize> = Vec::new();
    
        for mat in regex.find_iter(&buffer[..bytes_read]) {
            found.push(address + mat.start());
        }

        Ok((next_region, found))
    }

    pub fn scan_all(&mut self, handle: HANDLE, pattern: &str, return_multiple: bool) -> Result<Vec<usize>> {
        let mut next_region: usize = 0;
        let mut page_found: Vec<usize> = Vec::new();

        let mut found: Vec<usize> = Vec::new();

        while next_region < 0x07FFFFFFF0000 {
            (next_region, page_found) = self.scan_page_return_all(handle, next_region, pattern)?;

            if !page_found.is_empty() {
                found.extend_from_slice(&page_found);
            };

            if !return_multiple && !found.is_empty() {
                break
            }
        }

        Ok(found)
    }

    pub fn scan_entire_module(&mut self, handle: HANDLE, module: MODULEENTRY32, pattern: &str) -> Result<Vec<usize>> {
        let base_address = module.modBaseAddr as usize;
        let max_address = base_address + module.modBaseSize as usize;
        
        let mut page_address = base_address;
        let mut page_found: Vec<usize> = Vec::new();

        let mut found: Vec<usize> = Vec::new();

        while page_address < max_address {
            (page_address, page_found) = self.scan_page_return_all(handle, page_address, pattern)?;

            if !page_found.is_empty() {
                found.extend_from_slice(&page_found);
            }
        }

        Ok(found)
    }

    pub async fn pattern_scan(&mut self, pattern: &str, module_name_opt: Option<&str>, return_multiple: bool) -> Result<Vec<usize>> {
        let found_addresses = if let Some(module_name) = module_name_opt {
            let module_obj = match module_from_name(*self.process.lock().await as u32, module_name) {
                Some(module) => module,
                None => return Err(
                    anyhow!("\"{module_name}\" module not found.")
                )
            };

            let ptr = *self.process.lock().await;
            self.scan_entire_module(ptr, module_obj, pattern)?
        } else {
            let ptr = *self.process.lock().await;
            self.scan_all(ptr, pattern, return_multiple)?
        };

        if found_addresses.is_empty() {
            return Err(anyhow!("Pattern \"{pattern}\" failed. You most likely need to restart the client."))
        }

        if found_addresses.len() > 1 && !return_multiple {
            return Err(anyhow!("Get {} results for {pattern}", found_addresses.len()))
        }

        if return_multiple {
            return Ok(found_addresses)
        }

        if let Some(first_found) = found_addresses.first() {
            return Ok(vec![*first_found])
        }

        Ok(found_addresses)
    }

    pub async fn get_address_from_symbol(&mut self, module_name: &str, symbol_name: &str, module_dir_opt: Option<&str>, force_reload: bool) -> Result<usize> {
        let module_dir = match module_dir_opt {
            Some(dir) => dir,
            None => &get_system_directory(100)
        };

        let file_path = PathBuf::from(module_dir).join(PathBuf::from(module_name));
        if !file_path.exists() {
            return Err(anyhow!("No module named {module_name}"))
        }

        let file_path_str = match file_path.to_str() {
            Some(f) => f,
            None => return Err(anyhow!("Could not convert PathBuf \"{:#?}\"", file_path))
        };

        let symbols = self.get_symbols(file_path_str, force_reload)?;

        let symbol = match symbols.get(symbol_name) {
            Some(s) => s,
            None => return Err(anyhow!("No symbol named \"{symbol_name}\" in module \"{module_name}\""))
        };

        let module = match module_from_name(*self.process.lock().await as u32, module_name) {
            Some(module_obj) => module_obj,
            None => return Err(anyhow!("Could not get module \"{module_name}\" despite being in symbol table."))
        };

        return Ok(module.modBaseAddr as usize + *symbol as usize)
    }

    pub async fn allocate(&mut self, size: usize) -> Result<usize> {
        let result = unsafe {
            let allocated = VirtualAllocEx(
                *self.process.lock().await,
                null_mut(),
                size,
                MEM_COMMIT | MEM_RESERVE,
                 PAGE_READWRITE
            );

            if allocated.is_null() {
                return Err(anyhow!("Failed to allocate memory"))
            }

            allocated as usize
        };

        return Ok(result)
    }


    pub async fn free(&mut self, address: usize) -> Result<()> {
        let result = unsafe {
            VirtualFreeEx(
                *self.process.lock().await, 
                address as *mut c_void,
                0,
                MEM_RELEASE
            )
        };

        if result == 0 {
            return Err(anyhow!("Failed to free memory at address {:#x}", address))
        }

        Ok(())
    }


    pub async fn start_thread(&mut self, address: usize) -> Result<()> {
        // Create a new thread in the current process
        let thread_handle = unsafe {
            CreateRemoteThread(
                *self.process.lock().await,  // Handle to the current process
                null_mut(),       // Default security attributes
                0,                // Default stack size
                Some(std::mem::transmute::<*mut c_void, unsafe extern "system" fn(LPVOID) -> u32>(address as *mut c_void)),  // Thread function
                null_mut(),       // Argument to pass to the thread function
                0,                // Creation flags
                null_mut(),       // Pointer to receive the thread ID
            )
        };
    
        if thread_handle.is_null() || thread_handle == INVALID_HANDLE_VALUE {
            return Err(anyhow!("Failed to create thread"))
        }

        Ok(())
    }

    pub async fn read_bytes(&mut self, address: usize, size: usize) -> Result<Vec<u8>> {
        if !(0 < address && address <= 0x7FFFFFFFFFFFFFFF) {
            return Err(anyhow!("Address \"{:#x}\" out of bounds", address))
        }

        let mut buffer = vec![0u8; size];
        let mut bytes_read: usize = 0;

        let result = unsafe {
            ReadProcessMemory(
                *self.process.lock().await,
                address as *mut c_void,
                buffer.as_mut_ptr() as *mut _,
                size,
            &mut bytes_read)
        };

        if !self.is_running().await? && result == 0 {
            return Err(anyhow!("Client must be running to perform this action."))
        }

        if result == 0 {
            return Err(anyhow!("Unable to read memory at address \"{:#x}\".", address))
        }

        return Ok(buffer)
    }


    pub async fn write_bytes(&mut self, address: usize, mut value: Vec<u8>) -> Result<()> {
        let size = value.len();
        let mut bytes_written: usize = 0;

        let result = unsafe {
            WriteProcessMemory(
                *self.process.lock().await,
                address as *mut c_void,
                value.as_mut_ptr() as *mut _,
                size,
                &mut bytes_written,
            )
        };

        if !self.is_running().await? && result == 0 {
            return Err(anyhow!("Client must be running to perform this action."))
        }

        if result == 0 {
            return Err(anyhow!("Unable to write memory at address \"{:#x}\".", address))
        }

        Ok(())
    }


    pub async fn read_typed<T>(&mut self, address: usize) -> Result<T>
    where
        T: Sized + Pod + Zeroable,
    {
        let data_bytes = self.read_bytes(address, std::mem::size_of::<T>()).await?;

        let data: T = *from_bytes(&data_bytes);
        
        return Ok(data)
    }


    pub async fn write_typed<T>(&mut self, address: usize, value: T) -> Result<()>
    where
        T: Sized + Pod + Zeroable,
    {
        let value_bytes = bytes_of(&value);
        self.write_bytes(address, value_bytes.to_vec()).await?;

        Ok(())
    }
}