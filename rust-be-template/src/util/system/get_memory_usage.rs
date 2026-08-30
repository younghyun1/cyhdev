pub fn get_memory_usage() -> u64 {
    #[cfg(target_os = "linux")]
    {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        // Try to read MemAvailable from /proc/meminfo
        if let Ok(file) = File::open("/proc/meminfo") {
            let reader = BufReader::new(file);
            let mut total_mem = 0;
            let mut available_mem = 0;
            let mut found_total = false;
            let mut found_avail = false;

            for line in reader.lines() {
                if let Ok(line) = line {
                    if line.starts_with("MemTotal:") {
                        if let Some(val) = parse_meminfo_kb(&line) {
                            total_mem = val * 1024; // KB to Bytes
                            found_total = true;
                        }
                    } else if line.starts_with("MemAvailable:")
                        && let Some(val) = parse_meminfo_kb(&line)
                    {
                        available_mem = val * 1024; // KB to Bytes
                        found_avail = true;
                    }
                }
                if found_total && found_avail {
                    break;
                }
            }

            if found_total && found_avail {
                return total_mem.saturating_sub(available_mem);
            }
        }

        // Fallback to libc::sysinfo if /proc/meminfo fails or is missing MemAvailable
        use libc::{sysinfo, sysinfo as sysinfo_fn};
        use std::mem::MaybeUninit;

        unsafe {
            let mut info = MaybeUninit::<sysinfo>::uninit();
            if sysinfo_fn(info.as_mut_ptr()) == 0 {
                let info = info.assume_init();
                let total = info.totalram * info.mem_unit as u64;
                let free = info.freeram * info.mem_unit as u64;
                // Note: This fallback does not account for cache/buffers, so it reports higher usage.
                total.saturating_sub(free)
            } else {
                0
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        macos_memory_usage().unwrap_or(0)
    }
}

#[cfg(target_os = "macos")]
fn macos_memory_usage() -> Option<u64> {
    use std::mem::MaybeUninit;

    let mut statistics = MaybeUninit::<libc::vm_statistics64>::zeroed();
    let mut count = libc::HOST_VM_INFO64_COUNT;
    let status = unsafe {
        libc::host_statistics64(
            libc::mach_host_self(),
            libc::HOST_VM_INFO64,
            statistics.as_mut_ptr().cast::<libc::integer_t>(),
            &mut count,
        )
    };
    if status != libc::KERN_SUCCESS {
        return None;
    }

    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    let page_size = u64::try_from(page_size).ok()?;
    let statistics = unsafe { statistics.assume_init() };
    let used_pages = u64::from(statistics.active_count)
        + u64::from(statistics.inactive_count)
        + u64::from(statistics.wire_count)
        + u64::from(statistics.compressor_page_count);
    Some(used_pages.saturating_mul(page_size))
}

#[cfg(target_os = "linux")]
fn parse_meminfo_kb(line: &str) -> Option<u64> {
    // line format: "Key:       123456 kB"
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_memory_usage_nonzero_and_logs_time() {
        use std::time::Instant;
        let start = Instant::now();
        let usage = get_memory_usage();
        let duration = start.elapsed();
        println!(
            "get_memory_usage() took {:?} and returned {} bytes.",
            duration, usage
        );
    }
}
