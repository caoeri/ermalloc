use libc::*;

#[repr(C)]
#[derive(Copy, Clone)]
enum ErPolicyRaw {
    Nil,
    Redundancy,
}

#[repr(C)]
struct ErPolicyListRaw {
    policy: ErPolicyRaw,
    policy_data: *const c_void,
    er_list_policy_raw: *const ErPolicyListRaw,
}

