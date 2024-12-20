// kvm_bsd.rs

use std::sync::Mutex;
use std::ffi::CString;
use std::ptr;
use libc::{c_int, uint64_t, O_RDONLY};

extern "C" {
    fn kvm_open(
        execfile: *const i8,
        corefile: *const i8,
        swapfile: *const i8,
        flags: c_int,
        errstr: *const i8,
    ) -> *mut libc::c_void;
    fn kvm_getswapinfo(
        kd: *mut libc::c_void,
        kvmsw: *mut KvmSwap,
        maxswap: c_int,
        flags: c_int,
    ) -> c_int;
    fn kvm_close(kd: *mut libc::c_void) -> c_int;
}

#[repr(C)]
struct KvmSwap {
    ksw_total: i32,
    ksw_used: i32,
    ksw_free: i32,
    ksw_percent: i32,
    ksw_flags: i32,
}

pub struct Kvm {
    mu: Mutex<()>,
    has_err: bool,
}

impl Kvm {
    pub fn new() -> Self {
        Kvm {
            mu: Mutex::new(()),
            has_err: false,
        }
    }

    pub fn swap_used_pages(&self) -> Result<u64, String> {
        let _lock = self.mu.lock().unwrap();
        let mut value: uint64_t = 0;
        if unsafe { kvm_swap_used_pages(&mut value) } == -1 {
            self.has_err = true;
            return Err("couldn't get kvm stats".to_string());
        }
        Ok(value)
    }
}

unsafe fn kvm_swap_used_pages(out: *mut uint64_t) -> c_int {
    const TOTAL_ONLY: c_int = 1;

    let kd = kvm_open(ptr::null(), CString::new("/dev/null").unwrap().as_ptr(), ptr::null(), O_RDONLY, ptr::null());
    if kd.is_null() {
        return -1;
    }

    let mut current = KvmSwap {
        ksw_total: 0,
        ksw_used: 0,
        ksw_free: 0,
        ksw_percent: 0,
        ksw_flags: 0,
    };

    if kvm_getswapinfo(kd, &mut current, TOTAL_ONLY, 0) == -1 {
        kvm_close(kd);
        return -1;
    }

    if kvm_close(kd) != 0 {
        return -1;
    }

    *out = current.ksw_used as uint64_t;
    0
}