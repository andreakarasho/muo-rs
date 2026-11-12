//! A real memory figure for `GC.GetTotalMemory(false)` (admin gump, UOGateway): the process's resident set in bytes,
//! 0 where the host offers none.

#[cfg(windows)]
pub fn process_memory_bytes() -> i64 {
    #[repr(C)]
    struct Counters {
        cb: u32,
        page_faults: u32,
        peak_working_set: usize,
        working_set: usize,
        rest: [usize; 6],
    }
    unsafe extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(p: isize, c: *mut Counters, cb: u32) -> i32;
    }
    let mut c = Counters {
        cb: std::mem::size_of::<Counters>() as u32,
        page_faults: 0,
        peak_working_set: 0,
        working_set: 0,
        rest: [0; 6],
    };
    // SAFETY: `c` is a valid PROCESS_MEMORY_COUNTERS-sized buffer with `cb` set; the pseudo-handle needs no close.
    let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
    if ok != 0 { c.working_set as i64 } else { 0 }
}

#[cfg(not(windows))]
pub fn process_memory_bytes() -> i64 {
    // shortcut: assumes 4 KiB pages (/proc/self/statm counts pages); other hosts report 0.
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| s.split_whitespace().nth(1)?.parse::<i64>().ok())
        .map_or(0, |pages| pages * 4096)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reports_something_where_supported() {
        if cfg!(any(windows, target_os = "linux")) {
            assert!(super::process_memory_bytes() > 0);
        }
    }
}
