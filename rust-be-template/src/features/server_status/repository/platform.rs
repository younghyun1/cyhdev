//! Linux/macOS host sampling infrastructure.

pub async fn cpu_usage() -> f64 {
    let first = match tokio::task::spawn_blocking(cpu_ticks).await {
        Ok(Some(value)) => value,
        Ok(None) | Err(_) => return 0.0,
    };
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let second = match tokio::task::spawn_blocking(cpu_ticks).await {
        Ok(Some(value)) => value,
        Ok(None) | Err(_) => return 0.0,
    };
    let total = second.0.saturating_sub(first.0);
    let idle = second.1.saturating_sub(first.1);
    if total == 0 {
        0.0
    } else {
        total.saturating_sub(idle).saturating_mul(100) as f64 / total as f64
    }
}

#[cfg(target_os = "linux")]
fn cpu_ticks() -> Option<(u64, u64)> {
    use std::io::{BufRead, BufReader};
    let line = BufReader::new(std::fs::File::open("/proc/stat").ok()?)
        .lines()
        .next()?
        .ok()?;
    let values = line
        .split_whitespace()
        .skip(1)
        .take(8)
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if values.len() != 8 {
        return None;
    }
    let idle = match (values.get(3), values.get(4)) {
        (Some(idle), Some(iowait)) => idle.saturating_add(*iowait),
        _ => return None,
    };
    Some((values.into_iter().sum(), idle))
}

#[cfg(target_os = "macos")]
fn cpu_ticks() -> Option<(u64, u64)> {
    use std::mem::MaybeUninit;
    let mut info = MaybeUninit::<libc::host_cpu_load_info>::zeroed();
    let mut count = libc::HOST_CPU_LOAD_INFO_COUNT;
    let status = unsafe {
        libc::host_statistics(
            libc::mach_host_self(),
            libc::HOST_CPU_LOAD_INFO,
            info.as_mut_ptr().cast::<libc::integer_t>(),
            &mut count,
        )
    };
    if status != libc::KERN_SUCCESS {
        return None;
    }
    let ticks = unsafe { info.assume_init() }.cpu_ticks;
    let user = u64::from(ticks[libc::CPU_STATE_USER as usize]);
    let system = u64::from(ticks[libc::CPU_STATE_SYSTEM as usize]);
    let idle = u64::from(ticks[libc::CPU_STATE_IDLE as usize]);
    let nice = u64::from(ticks[libc::CPU_STATE_NICE as usize]);
    Some((user + system + idle + nice, idle))
}

pub fn total_memory() -> u64 {
    let pages = unsafe { libc::sysconf(libc::_SC_PHYS_PAGES) };
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    match (u64::try_from(pages), u64::try_from(page_size)) {
        (Ok(pages), Ok(page_size)) => pages.saturating_mul(page_size),
        _ => 0,
    }
}

pub fn memory_usage() -> u64 {
    #[cfg(target_os = "linux")]
    {
        linux_memory_usage()
    }
    #[cfg(target_os = "macos")]
    {
        macos_memory_usage().unwrap_or(0)
    }
}

#[cfg(target_os = "linux")]
fn linux_memory_usage() -> u64 {
    use std::io::{BufRead, BufReader};
    let file = match std::fs::File::open("/proc/meminfo") {
        Ok(file) => file,
        Err(_) => return linux_sysinfo_memory(),
    };
    let mut total = None;
    let mut available = None;
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let value = || line.split_whitespace().nth(1)?.parse::<u64>().ok();
        if line.starts_with("MemTotal:") {
            total = value();
        } else if line.starts_with("MemAvailable:") {
            available = value();
        }
        if total.is_some() && available.is_some() {
            break;
        }
    }
    match (total, available) {
        (Some(total), Some(available)) => total.saturating_sub(available).saturating_mul(1024),
        _ => linux_sysinfo_memory(),
    }
}

#[cfg(target_os = "linux")]
fn linux_sysinfo_memory() -> u64 {
    use std::mem::MaybeUninit;
    let mut info = MaybeUninit::<libc::sysinfo>::uninit();
    if unsafe { libc::sysinfo(info.as_mut_ptr()) } != 0 {
        return 0;
    }
    let info = unsafe { info.assume_init() };
    info.totalram
        .saturating_sub(info.freeram)
        .saturating_mul(u64::from(info.mem_unit))
}

#[cfg(target_os = "macos")]
fn macos_memory_usage() -> Option<u64> {
    use std::mem::MaybeUninit;
    let mut stats = MaybeUninit::<libc::vm_statistics64>::zeroed();
    let mut count = libc::HOST_VM_INFO64_COUNT;
    if unsafe {
        libc::host_statistics64(
            libc::mach_host_self(),
            libc::HOST_VM_INFO64,
            stats.as_mut_ptr().cast::<libc::integer_t>(),
            &mut count,
        )
    } != libc::KERN_SUCCESS
    {
        return None;
    }
    let page_size = u64::try_from(unsafe { libc::sysconf(libc::_SC_PAGESIZE) }).ok()?;
    let stats = unsafe { stats.assume_init() };
    let pages = u64::from(stats.active_count)
        + u64::from(stats.inactive_count)
        + u64::from(stats.wire_count)
        + u64::from(stats.compressor_page_count);
    Some(pages.saturating_mul(page_size))
}
