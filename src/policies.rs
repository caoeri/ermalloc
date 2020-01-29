extern crate lazy_static;

use std::collections::HashMap;
use std::ffi::c_void;

const MAX_POLICIES: usize = 10;

#[repr(u64)]
pub enum Policy {
    Redundancy(u32),
    ReedSolomon,
    Custom, // TODO: Make ths a function to arbitrary data
}

impl Policy {
    /// Enforces the given policy and returns the number of errors found
    ///
    /// # Arguments
    ///
    /// * `data` - A byte slice that holds the data to enforce the policy on
    fn enforce_policy(&self, data: &mut [u8]) -> u32 {
        let num_errors = 0;
        match self {
            Policy::Redundancy(num_copies) => println!("Num_copies: {}", num_copies),
            Policy::ReedSolomon => println!("Reed Solomon"),
            Policy::Custom => println!("Custom"),
        }
        num_errors + (data[0] as u32)
    }
}

#[repr(C)]
struct MetaData {
    policies: [Policy; MAX_POLICIES],
    length: u32,
}

#[repr(C)]
struct AllocBlock {
    meta: MetaData,
    data: [u8],
}

// lazy_static! {
#[cfg(not(light_weight))]
unsafe {static alloc_blocks: HashMap<*mut [u8], &mut AllocBlock> = HashMap::new();}
// }

impl AllocBlock {
    fn as_ptr(&mut self) -> *mut c_void {
        self.data.as_ptr() as *mut c_void
    }

    fn enforce_policy(&mut self) -> u32 {
        let mut num_errors = 0;
        for i in self.meta.policies.iter() {
            num_errors += i.enforce_policy(&mut self.data);
        }
        num_errors
    }

    // #[cfg(light_weight)]
    // fn from_ptr (ptr: *mut c_void) -> AllocBlock {
    //     let data_ptr = ptr as *mut u8;
    //     unsafe {data_ptr.offset(-1 *  std::mem::size_of<MetaData>())}
    // }

    // #[cfg(light_weight)]
    // fn from_ptr (ptr: *mut c_void) -> AllocBlock {
    //     let data_ptr = ptr as *mut u8;
    //     unsafe {data_ptr.offset(-1 *  std::mem::size_of<MetaData>())}
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let x = Policy::ReedSolomon;
        let mut data = [1];
        x.enforce_policy(&mut data);
        let y = Policy::Custom;
        y.enforce_policy(&mut data);

        let y = Policy::Redundancy(32);
        y.enforce_policy(&mut data);
    }
}
