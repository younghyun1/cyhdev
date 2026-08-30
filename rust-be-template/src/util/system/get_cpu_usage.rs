pub async fn get_cpu_usage() -> f64 {
    #[cfg(target_os = "linux")]
    {
        use std::{
            fs::File,
            io::{BufRead, BufReader},
            time::Duration,
        };

        fn read_proc_stat() -> Option<(u64, u64)> {
            let file = File::open("/proc/stat").ok()?;
            let reader = BufReader::new(file);
            let line = reader.lines().next()?.ok()?;

            let mut parts = line.split_whitespace();
            let _cpu = parts.next()?;
            let user = parts.next()?.parse::<u64>().ok()?;
            let nice = parts.next()?.parse::<u64>().ok()?;
            let system = parts.next()?.parse::<u64>().ok()?;
            let idle = parts.next()?.parse::<u64>().ok()?;
            let iowait = parts.next()?.parse::<u64>().ok()?;
            let irq = parts.next()?.parse::<u64>().ok()?;
            let softirq = parts.next()?.parse::<u64>().ok()?;
            let steal = parts.next()?.parse::<u64>().ok()?;
            // Guest times are already included in user/nice on modern kernels
            let _guest = parts.next()?.parse::<u64>().ok()?;
            let _guest_nice = parts.next()?.parse::<u64>().ok()?;

            let idle_all = idle + iowait;
            // Removed guest and guest_nice from total to avoid double counting
            let total = user + nice + system + idle + iowait + irq + softirq + steal;

            Some((total, idle_all))
        }

        let (total1, idle1) = match tokio::task::spawn_blocking(read_proc_stat).await {
            Ok(Some(vals)) => vals,
            _ => return 0.0,
        };

        tokio::time::sleep(Duration::from_millis(100)).await;

        let (total2, idle2) = match tokio::task::spawn_blocking(read_proc_stat).await {
            Ok(Some(vals)) => vals,
            _ => return 0.0,
        };

        let total_delta = total2.saturating_sub(total1);
        let idle_delta = idle2.saturating_sub(idle1);

        if total_delta == 0 {
            0.0
        } else {
            ((total_delta - idle_delta) as f64) * 100.0 / (total_delta as f64)
        }
    }

    #[cfg(target_os = "macos")]
    {
        macos_cpu_usage().await
    }
}

#[cfg(target_os = "macos")]
async fn macos_cpu_usage() -> f64 {
    use std::time::Duration;

    let (total_before, idle_before) =
        match tokio::task::spawn_blocking(read_macos_cpu_ticks).await {
            Ok(Some(ticks)) => ticks,
            Ok(None) | Err(_) => return 0.0,
        };
    tokio::time::sleep(Duration::from_millis(100)).await;
    let (total_after, idle_after) =
        match tokio::task::spawn_blocking(read_macos_cpu_ticks).await {
            Ok(Some(ticks)) => ticks,
            Ok(None) | Err(_) => return 0.0,
        };

    let total_delta = total_after.saturating_sub(total_before);
    let idle_delta = idle_after.saturating_sub(idle_before);
    if total_delta == 0 {
        0.0
    } else {
        total_delta
            .saturating_sub(idle_delta)
            .saturating_mul(100) as f64
            / total_delta as f64
    }
}

#[cfg(target_os = "macos")]
fn read_macos_cpu_ticks() -> Option<(u64, u64)> {
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

#[cfg(test)]
mod tests {
    use super::get_cpu_usage;
    use std::time::Instant;

    #[tokio::test]
    async fn test_get_cpu_usage() {
        let start = Instant::now();
        let usage = get_cpu_usage().await;
        let dur = start.elapsed();
        println!("CPU usage: {:.2}%", usage);
        println!("Elapsed time: {:?}", dur);
        assert!((0.0..=100.0).contains(&usage));
    }
}
