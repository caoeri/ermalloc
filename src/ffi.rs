use libc::*;

use std::ptr;

use crate::policies::Policy;

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

pub struct ErPolicyListNonNull {
    policy: ErPolicyRaw,
    policy_data: ptr::NonNull<c_void>,
    er_list_policy_raw: Option<ptr::NonNull<ErPolicyListNonNull>>,
}

impl From<ErPolicyListNonNull> for Policy {
    fn from(raw: ErPolicyListNonNull) -> Self {
        match raw.policy {
        ErPolicyRaw::Nil => Policy::Nil,
        ErPolicyRaw::Redundancy => {
            let ptr = raw.policy_data.clone().cast::<u32>();
            let num;
            unsafe {
                num = *(ptr.as_ptr());
            }
            Policy::Redundancy(num)
        },
        }
    }
}

#[no_mangle]
pub extern "C" fn er_malloc(size: size_t, policies: *const ErPolicyListRaw) -> *mut c_void {
    ptr::null::<c_void>() as *mut c_void
}
