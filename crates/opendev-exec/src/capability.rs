/// Resource limits for sandboxed processes.
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_bytes: Option<u64>,
    pub max_cpu_seconds: Option<u64>,
    pub max_open_fds: Option<u64>,
    pub max_file_size_bytes: Option<u64>,
    pub max_processes: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: None,
            max_cpu_seconds: None,
            max_open_fds: Some(1024),
            max_file_size_bytes: Some(100 * 1024 * 1024), // 100MB
            max_processes: Some(64),
        }
    }
}

/// Apply resource limits to the current process (called in pre_exec hook).
#[cfg(unix)]
pub fn apply_limits(limits: &ResourceLimits) {
    use libc::{rlimit, RLIMIT_AS, RLIMIT_CPU, RLIMIT_NOFILE, RLIMIT_FSIZE, RLIMIT_NPROC};

    unsafe {
        if let Some(max_mem) = limits.max_memory_bytes {
            let rlim = rlimit {
                rlim_cur: max_mem,
                rlim_max: max_mem,
            };
            libc::setrlimit(RLIMIT_AS, &rlim);
        }
        if let Some(max_cpu) = limits.max_cpu_seconds {
            let rlim = rlimit {
                rlim_cur: max_cpu,
                rlim_max: max_cpu,
            };
            libc::setrlimit(RLIMIT_CPU, &rlim);
        }
        if let Some(max_fds) = limits.max_open_fds {
            let rlim = rlimit {
                rlim_cur: max_fds,
                rlim_max: max_fds,
            };
            libc::setrlimit(RLIMIT_NOFILE, &rlim);
        }
        if let Some(max_fsize) = limits.max_file_size_bytes {
            let rlim = rlimit {
                rlim_cur: max_fsize,
                rlim_max: max_fsize,
            };
            libc::setrlimit(RLIMIT_FSIZE, &rlim);
        }
        if let Some(max_procs) = limits.max_processes {
            let rlim = rlimit {
                rlim_cur: max_procs,
                rlim_max: max_procs,
            };
            libc::setrlimit(RLIMIT_NPROC, &rlim);
        }
    }
}

#[cfg(not(unix))]
pub fn apply_limits(_limits: &ResourceLimits) {
    // Not supported on non-Unix platforms
}
