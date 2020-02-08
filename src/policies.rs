extern crate lazy_static;

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::c_void;
use std::marker::PhantomData;

use std::boxed::Box;
use std::mem::transmute;

const MAX_POLICIES: usize = 3;

#[repr(u64)]
#[derive(Copy, Clone)]
pub enum Policy {
    Nil,
    Redundancy(u32),
    // ReedSolomon,
    // Custom, // TODO: Make ths a function to arbitrary data
}

impl Policy {
    /// Enforces the given policy and returns the number of errors found
    ///
    /// # Arguments
    ///
    /// * `data` - A byte slice that holds the data to enforce the policy on
    fn enforce_policy(&self, block: &mut AllocBlock) -> u32 {
        let num_errors = 0;
        match self {
            Policy::Redundancy(num_copies) => println!("Num_copies: {}", num_copies),
            // Policy::ReedSolomon => println!("Reed Solomon"),
            // Policy::Custom => println!("Custom"),
            Policy::Nil => println!("Nil"),
        }
        num_errors
    }

    fn enforce_redundancy(&self, block: &mut AllocBlock, index: usize) -> u32 {
        let mut errors = 0;
        let mut vec = Vec::new();
        match self {
            Policy::Redundancy(num_copies) => {
                // Load bits
                for copy in 0..*num_copies {
                    let copy_idx = usize::try_from(copy).unwrap();
                    unsafe { vec.push(*block.ptr.add((copy_idx * block.length + index) as usize)) }
                }
                // Count bits
                let mut corrected = 0;
                for bit in 0..8 {
                    let mask = 1 << bit;
                    let mut count: [u32; 2] = [0, 0];

                    for copy in vec.iter() {
                        count[((copy & mask) >> bit) as usize] += 1;
                    }

                    if count[0] < count[1] {
                        corrected |= 1 << bit;
                        errors += count[0];
                    } else {
                        errors += count[1];
                    }
                }
                // Correct everything
                for copy in 0..*num_copies {
                    let copy_idx = usize::try_from(copy).unwrap();
                    unsafe {
                        *block.ptr.add((copy_idx * block.length + index) as usize) = corrected;
                    }
                }
            }
            _ => panic!("Tried to enforce redundancy on a non-redundant policy"),
        }
        errors
    }
}

#[repr(C)]
struct AllocBlock {
    policies: [Policy; MAX_POLICIES],
    length: usize,
    ptr: *mut u8,
}

impl Drop for AllocBlock {
    fn drop(&mut self) {
        let full_size: usize = AllocBlock::size_of(self.length, &self.policies);
        let res = Layout::from_size_align(full_size + std::mem::size_of::<AllocBlock>(), 16);

        match res {
            Ok(_val) => {
                let layout = res.unwrap();
                unsafe {
                    let ptr: *mut u8 = transmute(self as *mut AllocBlock);
                    dealloc(ptr, layout)
                };
            }
            Err(_e) => panic!("Invalid layout arguments"),
        }
    }
}

// #[cfg(light_weight)]
impl AllocBlock {
    fn as_ptr(&self) -> *mut c_void {
        self.ptr as *mut c_void
    }

    fn enforce_policy(&mut self) -> u32 {
        //let mut num_errors = 0;
        /*
        for i in self.policies.iter() {
            let slice = unsafe {
                std::slice::from_raw_parts_mut(
                    self.ptr,
                    AllocBlock::size_of(self.length, &self.policies),
                )
            };
            // num_errors += i.enforce_policy(slice);
        }
        */
        let pol = self.policies.clone();
        // can't pass &mut self to enforce_policy while also iterating over original policy array
        pol.iter().fold(0, |acc, i| acc + i.enforce_policy(self))
        //num_errors
    }

    fn size_of(desired_size: usize, policies: &[Policy; MAX_POLICIES]) -> usize {
        let mut full_size = desired_size;
        for p in policies {
            match p {
                Policy::Redundancy(num_copies) => {
                    full_size *= usize::try_from(*num_copies).unwrap()
                }
                _ => (),
            }
        }
        full_size
    }

    fn new<'a>(size: usize, policies: &[Policy; MAX_POLICIES]) -> &'a mut AllocBlock {
        let full_size: usize = AllocBlock::size_of(size, policies);
        let res = Layout::from_size_align(full_size + std::mem::size_of::<AllocBlock>(), 16);

        match res {
            Ok(_val) => {
                let layout = res.unwrap();

                let block_ptr: *mut u8 = unsafe { alloc(layout) };
                let block: &'a mut AllocBlock;

                unsafe {
                    block = std::mem::transmute(block_ptr);
                }

                // TODO: Initialize block here
                block.length = size;
                block.policies = *policies;
                block.ptr = unsafe { block_ptr.add(std::mem::size_of::<AllocBlock>()) };
                block
            }
            Err(_e) => panic!("Invalid layout arguments"),
        }
    }

    fn as_slice(&mut self) -> &mut [u8] {
        let full_size: usize = AllocBlock::size_of(self.length, &self.policies);
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr, full_size)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redundancy_check() {
        let r_policy = Policy::Redundancy(3);

        let block: &mut AllocBlock = AllocBlock::new(
            1,
            &[r_policy, Policy::Nil, Policy::Nil]
        );

        // Create errors
        // unsafe {
        //     *block.ptr.add(0) = 0b1111;
        //     *block.ptr.add(1) = 0b1010;
        //     *block.ptr.add(2) = 0b0000;
        // }
        let slice = block.as_slice();

        slice[0] = 0b1111;
        slice[1] = 0b1010;
        slice[2] = 0b0000;

        assert_eq!(r_policy.enforce_redundancy(block, 0), 4);

        for idx in 0..3 {
            unsafe {
                assert_eq!(*block.ptr.add(idx), 0b1010 as u8);
            }
        }
    }
}
