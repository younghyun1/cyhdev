//! Persistence-free host sample values.

#[derive(Clone, Copy)]
pub struct SystemInfo {
    pub cpu_usage: f64,
    pub memory_usage: u64,
}
