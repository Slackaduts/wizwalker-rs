pub mod memory;
pub mod utils;

#[cfg(test)]
mod tests {
    // use super::*;
    #[test]
    fn test() {
        println!("{}", crate::utils::get_system_directory(100))
    }
}
