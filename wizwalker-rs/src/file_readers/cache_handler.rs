use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
// use std::fs;
// use std::path::Path;
use std::path::PathBuf;
// use serde_json::Value;
// use serde_json::Error as SerdeError;
use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::utils::{get_cache_folder, get_wiz_install, parse_template_id_file};
use crate::file_readers::wad::Wad;

use encoding::{all::UTF_16LE, DecoderTrap, Encoding};

// Define a type alias for our nested HashMap
pub type TemplateIDMap = HashMap<String, i32>;
pub type WadCache = HashMap<String, TemplateIDMap>;
pub type LangcodeMap = HashMap<String, HashMap<String, String>>;

pub struct CacheHandler {
    wad_cache: Option<WadCache>,
    template_ids: Option<HashMap<i32, String>>,
    root_wad: Wad
}


impl CacheHandler {
    pub fn new() -> Self {
        Self {
            wad_cache: None,
            template_ids: None,
            root_wad: {
                let mut wad = Wad::new(&PathBuf::from("root"));
                let _ = match wad.from_game_data("root") {
                    Ok(()) => (),
                    Err(e) => {
                        println!("Error occured when refreshing WAD from game data: \"{}\"", e);
                    }
                };

                wad
                // wad.from_game_data("root")
            }
        }
    }

    fn install_location(&self, override_path: Option<&str>) -> Result<PathBuf> {
        let path = get_wiz_install(override_path)?;
        Ok(path)
    }

    fn cache_dir(&self) -> Result<PathBuf> {
        return match get_cache_folder() {
            Some(f) => Ok(f),
            None => {
                Err(anyhow!("Could not get cache folder."))
            }
        }
    }

    fn check_updated(&self, wad_cache: WadCache, wad_file: Wad, files: Vec<&str>) -> Result<Vec<PathBuf>> {
        // if self.wad_cache.is_none() {
        //     self.wad_cache = Some(self.get_wad_cache()?);
        // }

        let mut res: Vec<&str> = Vec::new();

        for file_name in files {
            let file_info = wad_file.get_file_info(file_name)?;

            // let wad_cache = match &self.wad_cache {
            //     Some(a) => a,
            //     None => {
            //         &self.get_wad_cache()?
            //     } 
            // };

            match wad_cache.get(&wad_file.name) {
                Some(wad_outer) => {
                    match wad_outer.get(file_name) {
                        Some(wad_inner) => {
                            if *wad_inner as usize != file_info.size {
                                res.push(file_name);
                                
                            }
                        },
                        None => {

                        }
                    }
                },
                None => {
                    println!("\"{file_name}\" has not updated from \"{}\"", file_info.size);
                }
            }
        }

        Ok(Vec::new())
    }


    fn cache(&mut self) -> Result<()> {
        let mut root_wad = Wad::new(&PathBuf::from("root"));
        root_wad.from_game_data("root")?;

        self.cache_template(root_wad)?;

        Ok(())
    }


    fn cache_template(&mut self, root_wad: Wad) -> Result<()> {
        if self.wad_cache.is_none() {
            self.wad_cache = Some(self.get_wad_cache()?);
        }

        let wad_cache = match &self.wad_cache {
            Some(a) => a,
            None => return Err(anyhow!("Wad Cache does not exist. This should never happen."))
        };

        let template_file = &self.check_updated(wad_cache.clone(), root_wad.clone(), vec!["TemplateManifest.xml"])?;

        let file_data = &root_wad.get_file("TemplateManifest.xml")?;

        if !template_file.clone().is_empty() {
            
            let parsed_template_ids = parse_template_id_file(file_data.to_owned())?;

            let mut file = File::open(self.cache_dir()?.join("template_ids.json"))?;

            let mut file_data = Vec::new();
            file.read_to_end(&mut file_data)?;

            // let json_data = serde_json::from_slice(&file_data.as_slice())?;
            let json_data_vec: Vec<u8> = serde_json::to_vec(&parsed_template_ids)?; // THIS IS PROBABLY FUCKING WRONG - SLACK

            file.write(&json_data_vec.as_slice())?;
        }

        Ok(())
    }

    fn get_template_ids(&mut self) -> Result<HashMap<i32, String>> {
        return match &self.template_ids {
            Some(t) => Ok(t.clone()),
            None => {                  
                let mut file = File::open(self.cache_dir()?.join("template_ids.json"))?;

                let mut file_data = Vec::new();
                file.read_to_end(&mut file_data)?;

                let json_data: HashMap<i32, String> = serde_json::from_slice(&file_data.as_slice())?;
                self.template_ids = Some(json_data.clone());
                Ok(json_data)
            }
        }
    }

    fn parse_lang_file(&self, file_data: Vec<u8>) -> Result<HashMap<String, HashMap<String, String>>> {
        let decoded_str = match UTF_16LE.decode(&file_data, DecoderTrap::Strict) {
            Ok(dec) => dec,
            Err(e) => {
                return Err(anyhow!("Parsing lang file failed with the following error: \"{e}\""))
            }
        };

        let file_lines: Vec<&str> = decoded_str.split("\r\n").collect();

        // Get the header and the rest of the lines
        let header = file_lines.get(0).ok_or(anyhow!("Header line missing"))?;
        let lines = &file_lines[1..];

        // Extract language name from the header
        let parts: Vec<&str> = header.split(':').collect();
        let lang_name = parts.get(1).ok_or(anyhow!("Language name missing"))?.trim();

        // Create the language mapping
        let mut lang_mapping = HashMap::new();
        for chunk in lines.chunks(3) {
            if let (Some(key), Some(value)) = (chunk.get(0), chunk.get(2)) {
                lang_mapping.insert((*key).to_string(), (*value).to_string());
            }
        }

        // Return the result
        let mut result = HashMap::new();
        result.insert(lang_name.to_string(), lang_mapping);

        Ok(result)
    }


    fn get_all_lang_file_names(&self, mut root_wad: Wad) -> Result<Vec<String>> {
        let mut lang_file_names: Vec<String> = Vec::new();

        for file_name in root_wad.names()? {
            if file_name.starts_with("Locales/English/") { lang_file_names.push(file_name); }
        }

        Ok(lang_file_names)
    }


    fn read_lang_file(&self, root_wad: Wad, lang_file: &str) -> Result<LangcodeMap> {
        let file_data = root_wad.get_file(lang_file)?;
        let parsed_lang = self.parse_lang_file(file_data)?;

        Ok(parsed_lang)
    }


    fn cache_lang_file(&self, root_wad: Wad, lang_file: &str) -> Result<()> {
        let lang_files = vec![lang_file];

        let wad_cache = self.wad_cache.clone().ok_or(anyhow!("No wad cache"))?;
        if self.check_updated(wad_cache, root_wad.clone(), lang_files)?.is_empty() {
            return Ok(());
        }

        let parsed_lang = self.read_lang_file(root_wad, lang_file)?;

        let mut lang_map = self.get_langcode_map()?;
        lang_map.extend(parsed_lang);

        let mut langmap_file = File::open(
            self.cache_dir()?.join(PathBuf::from("langmap.json"))
        )?;

        let json_data_vec: Vec<u8> = serde_json::to_vec(&lang_map)?; // THIS IS PROBABLY FUCKING WRONG - SLACK
        langmap_file.write(json_data_vec.as_slice())?;

        Ok(())
    }


    fn cache_lang_files(&mut self, root_wad: Wad) -> Result<()> {
        let lang_file_names = self.get_all_lang_file_names(root_wad.clone())?;

        let mut parsed_lang_map: LangcodeMap = LangcodeMap::new();
        let wad_cache = self.wad_cache.clone().ok_or(anyhow!("Could not get wad cache."))?;

        for file_name in lang_file_names {
            let files: Vec<&str>  = vec![&file_name];
            if self.check_updated(wad_cache.clone(), root_wad.clone(), files)?.is_empty() {
                continue
            }

            let parsed_lang = match self.read_lang_file(root_wad.clone(), &file_name) {
                Ok(p) => p,
                Err(_) => { continue }
            };


            parsed_lang_map.extend(parsed_lang);

            self.write_wad_cache()?;

            let mut lang_map = self.get_langcode_map()?;
            lang_map.extend(parsed_lang_map.clone());

            let mut langmap_file = File::open(
                self.cache_dir()?.join(PathBuf::from("langmap.json"))
            )?;
    
            let json_data_vec: Vec<u8> = serde_json::to_vec(&lang_map)?; // THIS IS PROBABLY FUCKING WRONG - SLACK
            langmap_file.write(json_data_vec.as_slice())?;
        }

        Ok(())
    }
 

    fn get_wad_cache(&mut self) -> Result<HashMap<String, HashMap<String, i32>>> {
        let mut wad_cache_file = File::open(self.cache_dir()?.join("wad_cache.data"))?;
        
        let mut data = Vec::new();
        wad_cache_file.read_to_end(&mut data)?;

        let mut wad_cache: HashMap<String, HashMap<String, i32>> = HashMap::new();

        if !data.is_empty() {
            let wad_cache_data: Value = serde_json::from_slice(data.as_slice())?;

            let wad_cache_map = wad_cache_data.as_object().ok_or(
                anyhow!("Unable to read wad cache for wad_cache.data")
            )?;

            for (k, v) in wad_cache_map.iter() {
                let i = match v.as_object() {
                    Some(i1) => i1,
                    None => return Err(anyhow!("Unable to read wad cache for wad_cache.data"))
                };
                
                wad_cache.insert(k.to_owned(), HashMap::new());

                for (k1, v1) in i {
                    match wad_cache.get_mut(k) {
                        Some(a) => {
                            let num = match v1.as_number() {
                                Some(n) => match n.as_i64() {
                                    Some(i) => i,
                                    None => continue
                                },
                                None => continue
                            };
                            a.insert(k1.to_string(), num as i32)
                        },
                        None => continue
                    };
                }
            } 
        }

        return Ok(wad_cache)
    }


    fn write_wad_cache(&mut self) -> Result<()> {
        let mut wad_cache_file = File::open(
            self.cache_dir()?.join(PathBuf::from("wad_cache.data"))
        )?;
        
        let mut data = Vec::new();
        wad_cache_file.read_to_end(&mut data)?;
        
        let json_data: Value = serde_json::from_slice(data.as_slice())?;
        let json_data_vec: Vec<u8> = serde_json::to_vec(&json_data)?; // THIS IS PROBABLY FUCKING WRONG - SLACK

        let file = match &self.root_wad.file {
            Some(f) => f,
            None => return Err(anyhow!("No file found."))
        };

        let mut file_lock = match file.lock() {
            Ok(l) => l,
            Err(e) => return Err(anyhow!("Acquiring wad file lock failed with the following error: \"{}\"", e))
        };

        file_lock.write(json_data_vec.as_slice())?;

        Ok(())
    }



    fn get_langcode_map(&self) -> Result<LangcodeMap> {
        let mut langmap_file = File::open(
            self.cache_dir()?.join(PathBuf::from("langmap.json"))
        )?;

        let mut data = Vec::new();
        langmap_file.read_to_end(&mut data)?;
        
        let json_data: LangcodeMap = serde_json::from_slice(data.as_slice())?;
        Ok(json_data)
    }

    fn cache_all_langcode_maps(&mut self) -> Result<()> {
        let root_wad = self.root_wad.clone();
        self.cache_lang_files(root_wad)?;

        Ok(())
    }


    fn get_template_name(&mut self, template_id: i32) -> Result<Option<String>> {
        let template_ids = self.get_template_ids()?;

        let template_name_opt = template_ids.get(&template_id).cloned();
        return Ok(template_name_opt)
    }

    fn get_langcode_name(&self, langcode: &str) -> Result<String> {
        let split_point = langcode.find("_").ok_or(anyhow!("Could not find \"_\" in langcode \"{langcode}\""))?;
        let lang_filename = &langcode[..split_point];
        let code = &langcode[split_point + 1..];

        let lang_files = self.get_all_lang_file_names(self.root_wad.clone())?;

        let mut cached = false;
        for filename in lang_files {
            if filename == format!("Locale/English/{lang_filename}.lang") {
                self.cache_lang_file(self.root_wad.clone(), &filename)?;
                cached = true;
                break
            }
        }

        if !cached {
            return Err(anyhow!("No lang file named {lang_filename}"))
        }

        let langcode_map = self.get_langcode_map()?;
        
        let lang_file = langcode_map.get(code).ok_or(anyhow!("No lang file named {lang_filename}"))?;

        let lang_name = lang_file.get(code).ok_or(anyhow!("No lang name with code {code}"))?;
        
        return Ok(lang_name.clone())
    }
}