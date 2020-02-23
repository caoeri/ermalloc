use libc::*;

use std::ptr;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;

use crate::policies::Policy;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum ErPolicyRaw {
    Nil,
    Redundancy,
}

#[derive(Debug, Copy, Clone)]
pub enum FfiError {
    PolicyValueUnknown,
    PolicyDataWasNull,
    MoreThanMaxPolicies,
}

impl fmt::Display for FfiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for FfiError {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErPolicyListRaw {
    policy: ErPolicyRaw,
    policy_data: *const c_void,
    er_list_policy_raw: *const ErPolicyListRaw,
}

impl ErPolicyListRaw {
    fn new(policy: ErPolicyRaw, policy_data: *const c_void, er_list_policy_raw: *const ErPolicyListRaw) -> Self {
        ErPolicyListRaw { policy, policy_data, er_list_policy_raw }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ErPolicyListNonNull {
    policy: ErPolicyRaw,
    policy_data: Option<ptr::NonNull<c_void>>,
    er_list_policy: Option<ptr::NonNull<ErPolicyListRaw>>,
}

impl ErPolicyListNonNull {
    fn new(policy: ErPolicyRaw, policy_data: Option<ptr::NonNull<c_void>>, er_list_policy: Option<ptr::NonNull<ErPolicyListRaw>>) -> Self {
        ErPolicyListNonNull { policy, policy_data, er_list_policy }
    }
}

impl Iterator for ErPolicyListNonNull {
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        match self.er_list_policy {
            None => None,
            Some(ptr) => {
                unsafe {
                    ErPolicyListNonNull::try_from(*ptr.as_ptr()).ok()
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.er_list_policy {
            None => (0, None),
            Some(_) => (1, None)
        }
    }
}

impl From<ErPolicyListNonNull> for Policy {
    fn from(raw: ErPolicyListNonNull) -> Self {
        match raw.policy {
        ErPolicyRaw::Nil => Policy::Nil,
        ErPolicyRaw::Redundancy => {
            let ptr = raw.policy_data.unwrap().clone().cast::<u32>();
            let num;
            unsafe {
                num = *(ptr.as_ptr());
            }
            Policy::Redundancy(num)
        },
        }
    }
}

impl TryFrom<ErPolicyListRaw> for ErPolicyListNonNull {
    type Error = FfiError;

    fn try_from(raw: ErPolicyListRaw) -> Result<Self, Self::Error> {
        let next;
        if raw.er_list_policy_raw.is_null() {
            next = None;
        } else {
            unsafe {
                next = Some(ptr::NonNull::new_unchecked(raw.er_list_policy_raw as *mut _));
            }
        }
        match raw.policy {
            ErPolicyRaw::Nil => {
                Ok(ErPolicyListNonNull::new(raw.policy, None, next))
            }
            ErPolicyRaw::Redundancy => {
                if raw.policy_data.is_null() {
                    Err(FfiError::PolicyDataWasNull)
                } else {
                    let policy_data = unsafe {
                        Some(ptr::NonNull::new_unchecked(raw.policy_data as *mut _))
                    };
                    Ok(ErPolicyListNonNull::new(raw.policy, policy_data, next))
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn er_malloc(size: size_t, policies: *const ErPolicyListRaw) -> *mut c_void {
    ptr::null::<c_void>() as *mut c_void
}

#[no_mangle]
pub extern "C" fn er_free(ptr: *const c_void)  {
    println!("hi");
}

#[no_mangle]
pub extern "C" fn er_calloc(nmemb: size_t, size: size_t, policies: *const ErPolicyListRaw) -> *mut c_void {
    ptr::null::<c_void>() as *mut c_void
}

#[no_mangle]
pub extern "C" fn er_realloc(ptr: *const c_void, size: size_t, policies: *const ErPolicyListRaw) -> *mut c_void {
    ptr::null::<c_void>() as *mut c_void
}

#[no_mangle]
pub extern "C" fn er_reallocarray(ptr: *const c_void, nmemb: size_t, size: size_t, policies: *const ErPolicyListRaw) -> *mut c_void {
    ptr::null::<c_void>() as *mut c_void
}

