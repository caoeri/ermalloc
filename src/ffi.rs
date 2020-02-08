use libc::*;

use std::ptr;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum ErPolicyRaw {
    Nil,
    Redundancy,
}

#[repr(C)]
pub struct ErPolicyListRaw {
    policy: ErPolicyRaw,
    policy_data: *const c_void,
    er_list_policy_raw: *const ErPolicyListRaw,
}

#[no_mangle]
pub extern "C" fn er_malloc(size: size_t, policies: *const ErPolicyListRaw) -> *mut c_void {
    ptr::null::<c_void>() as *mut c_void
}
