use std::{collections::HashMap, fmt::Debug, sync::Arc, future::Future};
use anyhow::{Result, anyhow};
use tokio::sync::Mutex;
use winapi::um::winnt::HANDLE;

use super::memory_reader::MemoryReader;


#[derive(Debug)]
pub struct MemoryHook {
    hook_handler: Arc<Mutex<HANDLE>>,
    memory_reader: MemoryReader,
    hook_cache: HashMap<String, usize>,
    jump_original_bytecode: Vec<u8>,
 
    hook_address: usize,
    jump_address: usize,
 
    jump_bytecode: Vec<u8>,
    hook_bytecode: Vec<u8>,

    allocated_addresses: Vec<usize>
}

impl MemoryHook {
    pub fn new(hook_handler: HANDLE) -> Self {
        MemoryHook {
            hook_handler: Arc::new(Mutex::new(hook_handler)),
            memory_reader: MemoryReader::new(hook_handler),
            hook_cache: HashMap::new(),
            jump_original_bytecode: Vec::new(),
            hook_address: 0x0,
            hook_bytecode: Vec::new(),
            jump_address: 0x0,
            jump_bytecode: Vec::new(),
            allocated_addresses: Vec::new()
        }
    }
}

pub trait WizWalkerHook {
    fn is_cached(self, name: &str) -> bool;

    fn cache(&mut self, name: &str, value: usize);

    fn get_cached(self, name: &str) -> Option<usize>;

    fn alloc(&mut self, size: usize) -> impl Future<Output = Result<usize>>;

    fn prehook(&self) -> impl Future<Output = ()>;

    fn posthook(&self) -> impl Future<Output = ()>;

    fn get_jump_address(&mut self, pattern: &str, module: Option<&str>) -> impl Future<Output = Result<usize>>;

    fn get_hook_address(&mut self, size: usize) -> impl Future<Output = Result<usize>>;

    fn get_jump_bytecode(&self) -> impl Future<Output = Result<Vec<u8>>>;

    fn get_hook_bytecode(&self) -> impl Future<Output = Result<Vec<u8>>>;

    fn get_pattern(&self) -> impl Future<Output = Result<(String, String)>>;

    fn hook(&mut self) -> impl Future<Output = Result<()>>;

    fn unhook(&mut self) -> impl Future<Output = Result<()>>;
}

impl WizWalkerHook for MemoryHook {
    fn is_cached(self, name: &str) -> bool {
        return self.hook_cache.contains_key(name)
    }

    fn cache(&mut self, name: &str, value: usize) {
        self.hook_cache.insert(name.to_string(), value);
    }

    fn get_cached(self, name: &str) -> Option<usize> {
        return self.hook_cache.get(name).copied()
    }

    async fn alloc(&mut self, size: usize) -> Result<usize> {
        let addr = self.memory_reader.allocate(size).await?;
        self.allocated_addresses.push(addr);
        return Ok(addr)
    }

    async fn prehook(&self) {
        unimplemented!()
    }

    async fn posthook(&self) {
        unimplemented!()
    }

    async fn get_jump_address(&mut self, pattern: &str, module: Option<&str>) -> Result<usize> {
        let jump_addresses = self.memory_reader.pattern_scan(pattern, module, false).await?;
        return match jump_addresses.first() {
            Some(addr) => Ok(*addr),
            None => return Err(anyhow!("Could not find jump address. Pattern \"{pattern}\"."))
        }
    }


    async fn get_hook_address(&mut self, size: usize) -> Result<usize> {
        return self.memory_reader.allocate(size).await
    }

    async fn get_jump_bytecode(&self) -> Result<Vec<u8>> {
        unimplemented!()
    }

    async fn get_hook_bytecode(&self) -> Result<Vec<u8>> {
        unimplemented!()
    }

    async fn get_pattern(&self) -> Result<(String, String)> {
        unimplemented!()
    }

    async fn hook(&mut self) -> Result<()> {
        let (pattern, module) = self.get_pattern().await?;
        
        let jump_address = self.get_jump_address(&pattern, Some(&module)).await?;
        let hook_address = self.get_hook_address(50).await?;
    
        let hook_bytecode = self.get_hook_bytecode().await?;
        let jump_bytecode = self.get_jump_bytecode().await?;
     
        let jump_original_bytecode = self.memory_reader.read_bytes(
            jump_address, jump_bytecode.len()
        ).await?;
    
        // Use local variables to temporarily hold values
        self.jump_address = jump_address;
        self.hook_address = hook_address;
        self.hook_bytecode = hook_bytecode;
        self.jump_bytecode = jump_bytecode;
        self.jump_original_bytecode = jump_original_bytecode;
    
        self.prehook().await;
    
        self.memory_reader.write_bytes(self.hook_address, self.hook_bytecode.clone()).await?;
        self.memory_reader.write_bytes(self.jump_address, self.jump_bytecode.clone()).await?;

        self.posthook().await;
    
        Ok(())
    }


    async fn unhook(&mut self) -> Result<()> {
        let bytecode = self.jump_original_bytecode.clone();
        self.memory_reader.write_bytes(self.jump_address, bytecode).await?;
        
        for addr in self.allocated_addresses.clone() {
            self.memory_reader.free(addr).await?;
        }

        Ok(())
    }
}