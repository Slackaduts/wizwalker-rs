use anyhow::{Result, anyhow};
// use wizwalker_macros::memory_hook;


use std::collections::HashMap;
use winapi::um::winnt::HANDLE;

use super::memory_reader::WizWalkerMemoryReader;



pub struct MemoryHook {
    process: HANDLE,
    symbol_table: HashMap<String, HashMap<String, u32>>,
    hook_handler: HANDLE,
    hook_cache: HashMap<String, usize>,
    jump_original_bytecode: Vec<u8>,
    jump_bytecode: Vec<u8>,
    hook_address: usize,
    jump_address: usize,
    hook_bytecode: Vec<u8>,
    allocated_addresses: Vec<usize>,
}

impl MemoryHook {
    pub fn new(hook_handler: HANDLE) -> Self {
        Self {
            process: hook_handler,
            symbol_table: HashMap::new(),
            hook_handler: hook_handler,
            hook_cache: HashMap::new(),
            jump_original_bytecode: Vec::new(),
            jump_bytecode: Vec::new(),
            hook_address: 0x0,
            jump_address: 0x0,
            hook_bytecode: Vec::new(),
            allocated_addresses: Vec::new(),
        }
    }
}

impl WizWalkerMemoryReader for MemoryHook {
    fn process(&self) -> HANDLE {
        self.process
    }

    fn symbol_table(&mut self) -> &mut HashMap<String, HashMap<String, u32>> {
        &mut self.symbol_table
    }
}


impl WizWalkerMemoryHook for MemoryHook {
    fn hook_handler(&self) -> HANDLE {
        self.hook_handler
    }
    fn hook_cache(&mut self) -> &mut HashMap<String, usize> {
        &mut self.hook_cache
    }
    fn jump_original_bytecode(&mut self) -> &mut Vec<u8> {
        &mut self.jump_original_bytecode
    }
    fn jump_bytecode(&mut self) -> &mut Vec<u8> {
        &mut self.jump_bytecode
    }
    fn hook_address(&mut self) -> &mut usize {
        &mut self.hook_address
    }
    fn jump_address(&mut self) -> &mut usize {
        &mut self.jump_address
    }
    fn hook_bytecode(&mut self) -> &mut Vec<u8> {
        &mut self.hook_bytecode
    }
    fn allocated_addresses(&mut self) -> &mut Vec<usize> {
        &mut self.allocated_addresses
    }
}


pub trait WizWalkerMemoryHook: WizWalkerMemoryReader {
    fn hook_handler(&self) -> HANDLE;
    fn hook_cache(&mut self) -> &mut HashMap<String, usize>;
    fn jump_original_bytecode(&mut self) -> &mut Vec<u8>;
    fn jump_bytecode(&mut self) -> &mut Vec<u8>;
    fn hook_address(&mut self) -> &mut usize;
    fn jump_address(&mut self) -> &mut usize;
    fn hook_bytecode(&mut self) -> &mut Vec<u8>;
    fn allocated_addresses(&mut self) -> &mut Vec<usize>;

    fn is_cached(&mut self, name: &str) -> bool {
        return self.hook_cache().contains_key(name)
    }

    fn cache(&mut self, name: &str, value: usize, hook_cache: &mut HashMap<String, usize>) {
        // Discarding this value may have unintended consequences. -Slack
        let _ = hook_cache.insert(name.to_string(), value);
    }

    fn get_cached(&mut self, name: &str) -> Option<usize> {
        return self.hook_cache().get(name).copied()
    }

    fn alloc(&mut self, size: usize) -> Result<usize> {
        let addr = self.allocate(size)?;
        self.allocated_addresses().push(addr);
        return Ok(addr)
    }

    fn prehook(&self) {
        unimplemented!()
    }

    fn posthook(&self) {
        unimplemented!()
    }

    fn get_jump_address(&mut self, pattern: &str, module: Option<&str>) -> Result<usize> {
        let jump_addresses = self.pattern_scan(pattern, module, false)?;
        return match jump_addresses.first() {
            Some(addr) => Ok(*addr),
            None => return Err(anyhow!("Could not find jump address. Pattern \"{pattern}\"."))
        }
    }

    fn get_hook_address(&mut self, size: usize) -> Result<usize> {
        return self.allocate(size)
    }

    fn get_jump_bytecode(&self) -> Result<Vec<u8>> {
        unimplemented!()
    }

    fn get_hook_bytecode(&self) -> Result<Vec<u8>> {
        unimplemented!()
    }

    fn get_pattern(&self) -> Result<(String, String)> {
        unimplemented!()
    }

    fn hook(&mut self) -> Result<()> {
        let (pattern, module) = self.get_pattern()?;
        
        *self.jump_address() = self.get_jump_address(&pattern, Some(&module))?;
        *self.hook_address() = self.get_hook_address(50)?;
    
        *self.hook_bytecode() = self.get_hook_bytecode()?;
        *self.jump_bytecode() = self.get_jump_bytecode()?;

        let jump_addr = *self.jump_address();
        let jump_b_size = self.jump_bytecode().len();
     
        *self.jump_original_bytecode() = self.read_bytes(
            jump_addr, 
        jump_b_size,
        )?;
    
        self.prehook();

        let hook_addr = *self.hook_address();
        let jump_bytec = self.jump_bytecode().clone();
        let hook_bytec = self.hook_bytecode().clone();
    
        self.write_bytes(hook_addr, hook_bytec)?;
        self.write_bytes(jump_addr, jump_bytec)?;

        self.posthook();
    
        Ok(())
    }

    fn unhook(&mut self) -> Result<()> {
        let jump_addr = self.jump_address().clone();
        let jump_original_bytec = self.jump_original_bytecode().clone();
        let allocated_addrs = self.allocated_addresses().clone();

        self.write_bytes(jump_addr, jump_original_bytec)?;
        
        for addr in allocated_addrs {
            self.free(addr)?;
        }

        Ok(())
    }
}




// pub struct AutoBotBaseHook {
//     process: HANDLE,
//     symbol_table: HashMap<String, HashMap<String, u32>>,
//     hook_handler: HANDLE,
//     hook_cache: HashMap<String, usize>,
//     jump_original_bytecode: Vec<u8>,
//     jump_bytecode: Vec<u8>,
//     hook_address: usize,
//     jump_address: usize,
//     hook_bytecode: Vec<u8>,
//     allocated_addresses: Vec<usize>,
// }

// impl AutoBotBaseHook {
//     pub fn new(hook_handler: HANDLE) -> Self {
//         Self {
//             process: hook_handler,
//             symbol_table: HashMap::new(),
//             hook_handler: hook_handler,
//             hook_cache: HashMap::new(),
//             jump_original_bytecode: Vec::new(),
//             jump_bytecode: Vec::new(),
//             hook_address: 0x0,
//             jump_address: 0x0,
//             hook_bytecode: Vec::new(),
//             allocated_addresses: Vec::new(),
//         }
//     }
// }

// impl WizWalkerMemoryReader for AutoBotBaseHook {
//     fn process(&self) -> HANDLE {
//         self.process
//     }

//     fn symbol_table(&mut self) -> &mut HashMap<String, HashMap<String, u32>> {
//         &mut self.symbol_table
//     }
// }

// impl WizWalkerMemoryHook for AutoBotBaseHook {
//     fn hook_handler(&self) -> HANDLE {
//         self.hook_handler
//     }
//     fn hook_cache(&mut self) -> &mut HashMap<String, usize> {
//         &mut self.hook_cache
//     }
//     fn jump_original_bytecode(&mut self) -> &mut Vec<u8> {
//         &mut self.jump_original_bytecode
//     }
//     fn jump_bytecode(&mut self) -> &mut Vec<u8> {
//         &mut self.jump_bytecode
//     }
//     fn hook_address(&mut self) -> &mut usize {
//         &mut self.hook_address
//     }
//     fn jump_address(&mut self) -> &mut usize {
//         &mut self.jump_address
//     }
//     fn hook_bytecode(&mut self) -> &mut Vec<u8> {
//         &mut self.hook_bytecode
//     }
//     fn allocated_addresses(&mut self) -> &mut Vec<usize> {
//         &mut self.allocated_addresses
//     }
// }



// pub trait WizWalkerAutoBotBaseHook: WizWalkerMemoryHook {
//     fn alloc(&mut self, size: usize) -> Result<usize> {
//         self.hook_handler().
//     }
// }