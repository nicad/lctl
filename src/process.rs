//! libc calls: process start times (read from the kernel the way ps(1) does) and local time.

use std::ffi::CStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// When process `pid` started; `None` if there is no such process.
///
/// Uses `sysctl kern.proc.pid.<pid>`: unlike `proc_pidinfo`, it works for other users'
/// processes (root daemons) without root.
pub fn start_time(pid: u32) -> Option<SystemTime> {
    let mut mib = [libc::CTL_KERN, libc::KERN_PROC, libc::KERN_PROC_PID, pid.try_into().ok()?];
    // struct kinfo_proc is 648 bytes on 64-bit macOS
    let mut info = [0u8; 1024];
    let mut size = info.len();
    // SAFETY: `info` is writable for `size` bytes and sysctl writes at most that many.
    let rc = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as libc::c_uint,
            info.as_mut_ptr().cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if rc != 0 || size == 0 {
        return None;
    }
    // kinfo_proc starts with kp_proc.p_starttime, a struct timeval whose tv_sec is an i64
    let seconds = i64::from_ne_bytes(info[..8].try_into().ok()?);
    UNIX_EPOCH.checked_add(Duration::from_secs(seconds.try_into().ok()?))
}

/// `time` in the local timezone, formatted by strftime(3).
pub fn local_time(time: SystemTime, format: &CStr) -> String {
    let seconds = time.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()) as libc::time_t;
    let mut buf = [0u8; 64];
    // SAFETY: `tm` is plain data that localtime_r fills in; strftime writes at most
    // `buf.len()` bytes and returns how many it wrote (0 when the result doesn't fit).
    let len = unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&seconds, &mut tm).is_null() {
            return String::new();
        }
        libc::strftime(buf.as_mut_ptr().cast(), buf.len(), format.as_ptr(), &tm)
    };
    String::from_utf8_lossy(&buf[..len]).into_owned()
}
