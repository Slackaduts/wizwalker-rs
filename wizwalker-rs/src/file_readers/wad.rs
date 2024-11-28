use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::fs::{File, create_dir};
use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex};
use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;

use crate::utils::get_wiz_install;


#[derive(Clone)]
pub struct WadFileInfo {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub is_zip: bool,
    pub crc: i32,
    pub unzipped_size: usize
}


impl WadFileInfo {
    pub fn new(name: String, offset: usize, size: usize, is_zip: bool, crc: i32, unzipped_size: usize) -> Self {
        Self {
            name: name,
            offset: offset,
            size: size,
            is_zip: is_zip,
            crc: crc,
            unzipped_size: unzipped_size
        }
    }
}


#[derive(Clone)]
pub struct Wad {
    pub file: Option<Arc<Mutex<File>>>,
    pub file_list: Vec<WadFileInfo>,
    pub file_path: PathBuf,
    pub refreshed_once: bool,
    pub name: String
}


impl Wad {
    // Constructor to create a new instance of MyFile
    pub fn new(path: &PathBuf) -> Self {
        Self {
            // file: Some(
                // Arc::new(Mutex::new(std::fs::File::open(path)?))
            // ),
            file: None,
            file_list: Vec::new(),
            file_path: path.to_path_buf(),
            refreshed_once: false,
            name: {
                match path.file_stem() {
                    Some(s) => s.to_string_lossy().to_string(),
                    None => path.to_string_lossy().to_string()
                }
            }
        }
    }

    pub fn from_game_data(&mut self, name: &str) -> Result<()> {
        let mut new_name = name.to_owned();

        if !new_name.ends_with(".wad") {
            new_name.push_str(".wad");
        }
        
        let mut new_path = get_wiz_install(None)?;
        new_path.push("Data");
        new_path.push("GameData");
        new_path.push(new_name);

        if !new_path.exists() {
            return Err(anyhow!("Unable to find game data wad at path \"{}\" on system.", new_path.display()))
        }

        self.file = None;
        self.file_path = new_path;
        self.refreshed_once = false;
        self.name = match self.file_path.file_stem() {
            Some(s) => s.to_string_lossy().to_string(),
            None => self.file_path.to_string_lossy().to_string()
        };

        self.refresh_journal()?;

        Ok(())
    }

    pub fn open(&mut self) -> Result<()> {
        self.file = Some(Arc::new(Mutex::new(File::open(&self.file_path)?)));

        self.refresh_journal()?;
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        // drop(self.file);
        self.file = None;
        Ok(())
    }

    pub fn size(&mut self) -> Result<u64> {
        if self.file.is_none() {
            self.open()?
        }

        let file = match &self.file {
            Some(f) => f,
            None => return Err(anyhow!("No file found."))
        };

        let file_lock = match file.lock() {
            Ok(l) => l,
            Err(e) => return Err(anyhow!("Acquiring wad file lock failed with the following error: \"{}\"", e))
        };
        Ok(file_lock.metadata()?.len())
    }

    pub fn names(&mut self) -> Result<Vec<String>> {
        if self.file.is_none() {
            self.open()?
        }

        return Ok(self.file_list.iter().map(
                    |f| f.name.clone()
                ).collect())
    }

    pub fn refresh_journal(&mut self) -> Result<()> {
        if self.refreshed_once { return Ok(()) }

        let file = &self.file.clone().ok_or(anyhow!("Call open() on this wad before using this method."))?;

        let mut file_lock = match file.lock() {
            Ok(l) => l,
            Err(e) => return Err(anyhow!("Acquiring wad file lock failed with the following error: \"{}\"", e))
        };
        
        file_lock.seek(std::io::SeekFrom::Start(5))?;
        let version = file_lock.read_i32::<LittleEndian>()?;
        let file_num = file_lock.read_i32::<LittleEndian>()?;

        if version >= 2 as i32 {
            let mut buffer = [0u8; 1]; // Buffer to hold the single byte
            file_lock.read_exact(&mut buffer)?;
        }

        for _ in 0..file_num {
            let offset = file_lock.read_i32::<LittleEndian>()?;
            let size = file_lock.read_i32::<LittleEndian>()?;
            let zsize = file_lock.read_i32::<LittleEndian>()?;
            let mut is_zip_buf = [0u8; 1];
            file_lock.read_exact(&mut is_zip_buf)?;
            let is_zip: bool = is_zip_buf[0] != 0;
            let crc = file_lock.read_i32::<LittleEndian>()?;
            let name_length = file_lock.read_i32::<LittleEndian>()? as u8;

            let mut name_buffer = vec![0u8, name_length];
            file_lock.read_exact(&mut name_buffer)?;

            let name = String::from_utf8(name_buffer)?;

            self.file_list.push(
                WadFileInfo::new(name, offset as usize, size as usize, is_zip, crc, zsize as usize)
            );
        }

        Ok(())
    }


    pub fn get_file(&self, name: &str) -> Result<Vec<u8>> {
        let mut maybe_target_file: Option<&WadFileInfo> = None;
        for file in &self.file_list {
            if file.name == name { maybe_target_file = Some(file) }
        }

        let target_file = maybe_target_file.ok_or(anyhow!("File \"{name}\" not found."))?;

        let file = match &self.file {
            Some(f) => f.lock(),
            None => return Err(anyhow!("No file found."))
        };

        let mut file_lock = match file {
            Ok(l) => l,
            Err(e) => return Err(anyhow!("Acquiring wad file lock failed with the following error: \"{}\"", e))
        };

        file_lock.seek(std::io::SeekFrom::Start(target_file.size as u64))?;
        
        let mut raw_data_buf = [0u8, target_file.size as u8];
        file_lock.read_exact(&mut raw_data_buf)?;

        let mut data = Vec::new();

        if target_file.is_zip {
            let mut decoder = ZlibDecoder::new(raw_data_buf.as_slice());
            decoder.read_to_end(&mut data)?;
        } else {
            data = raw_data_buf.to_vec();
        }

        Ok(data)
    }


    pub fn get_file_info(&self, name: &str) -> Result<WadFileInfo> {
        self.file.clone().ok_or(anyhow!("Call open() on this wad before using this method."))?;

        let mut maybe_target_file: Option<&WadFileInfo> = None;
        for file in &self.file_list {
            if file.name == name { maybe_target_file = Some(file) }
        }

        let target_file = match maybe_target_file {
            Some(t) => t,
            None => return Err(anyhow!("File \"{name}\" not found."))
        };

        return Ok(target_file.clone())
    }


    pub fn unarchive(&mut self, path: &PathBuf) -> Result<()> {
        if !path.exists() { return Err(anyhow!("\"{}\" does not exist.", path.display())) }
        if !path.is_dir() { return Err(anyhow!("\"{}\" does not a directory.", path.display())) }

        for file in &self.file_list {
            let dirs: Vec<&str> = file.name.split("/").collect();

            if dirs.len() != 1 {
                let mut current = path.to_owned();
                for next_dir in &dirs[..dirs.len() - 1] {
                    let new_curr = current.join(next_dir);
                    current = new_curr;
                    if current.exists() { continue; }

                    create_dir(current.clone())?;
                }
            }

            let file_path = path.join(PathBuf::from(file.name.to_owned()));
            let file_data = self.get_file(&file.name)?;

            // This may cause unintended consequences. We should be waiting for when the file becomes available. - Slack
            let mut fp = File::open(file_path)?;
            fp.write(&file_data)?;
        }

        Ok(())
    }


    pub fn from_directory(&self, _path: &PathBuf) -> Result<()> {
        unimplemented!()
    }
}