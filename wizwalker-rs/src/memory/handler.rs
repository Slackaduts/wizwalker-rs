use crate::memory::memory_reader::WizWalkerMemoryReader;
use std::sync::{Arc, Mutex};

const AUTOBOT_PATTERN: &[u8] = &[
    0x48, 0x8B, 0xC4, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57,
    0x48, // ...
    0x48, // ...
    0x48, 0x89, 0x58, 0x10, 0x48, 0x89, 0x70, 0x18, 0x48, 0x89, 0x78, 0x20,
    0x48, 0x33, 0xC4, // ...
    0x4C, 0x8B, 0xE9, // .......
    0x80, // ...
    0x0F,
];

const AUTOBOT_SIZE: usize = 3900;

pub trait WizWalkerHookHandler: WizWalkerMemoryReader {
    fn autobot_address(&mut self) -> Arc<Mutex<&mut usize>>;
    fn original_autobot_bytes(&mut self) -> Arc<Mutex<&mut Vec<u8>>>;
    fn autobot_pos(&mut self) -> &mut usize;
    // fn active_hooks
}

// TODO: THIS ENTIRE FUCKING FILE