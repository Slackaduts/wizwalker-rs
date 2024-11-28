use winapi::um::winnt::HANDLE;
// use crate::memory::hooks::WizWalkerMemoryHook;

pub trait WizWalkerClient {
    fn window_handle(&self) -> HANDLE;
    fn hook_handler(&mut self) -> &Self;
    // fn cache_handler 
}