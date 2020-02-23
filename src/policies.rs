extern crate lazy_static;
extern crate reed_solomon;

use std::alloc::{alloc, alloc_zeroed, dealloc, realloc, Layout};
use std::convert::TryFrom;
use std::ffi::c_void;
use std::iter::Iterator;
use std::mem::transmute;

use reed_solomon::{Buffer, Decoder, Encoder};

pub const MAX_POLICIES: usize = 3;

#[repr(u64)]
#[derive(Copy, Clone)]
pub enum Policy {
    Nil,
    Redundancy(u32),
    ReedSolomon(u32),
    // Custom, // TODO: Make ths a function to arbitrary data
}

// TODO:
// Cleaner API
// Support Create, Read, Update, Delete
// Full alloc, realloc, dealloc
// Proper warnings for poor allocations
// Cleaner interface for size propagation upwards
// Interface: is_corrupted, apply, correct

// TODO:
// Is corrupted?
// Create definitions like buffer (data + ecc) and data
// - correct (verify redundant bits assert no errors) & apply (create redundant bits)

fn correct_bits_redundant(buffer: &mut [u8], n_copies: usize, index: usize) -> u32 {
    let mut errors = 0;
    if buffer.len() % n_copies != 0 {
        panic!("Buffer is not divisible by the number of redundant copies")
    }
    let data_len = buffer.len() / n_copies;

    let copied_bytes: Vec<_> = (0..n_copies)
        .map(|i| buffer[i * data_len + index])
        .collect();

    // Count bits
    let mut corrected: u8 = 0;
    for bit in 0..8 {
        let mask = 1 << bit;
        let mut count: [u32; 2] = [0, 0];

        for byte in copied_bytes.iter() {
            count[((byte & mask) >> bit) as usize] += 1;
        }

        if count[0] < count[1] {
            corrected |= 1 << bit;
            errors += count[0];
        } else {
            errors += count[1];
        }
    }
    // Correct everything
    for copy in 0..n_copies {
        buffer[copy * data_len + index] = corrected;
    }

    errors
}

impl Policy {

    /// From the buffer return (data, ecc)
    fn split_buffer_mut<'a>(&self, buffer: &'a mut [u8]) -> (&'a mut [u8], &'a mut [u8]) {
        let len = buffer.len();
        match self {
            Policy::Redundancy(n_copies) => {
                if len % (*n_copies as usize) != 0 {
                    panic!("Redundancy: Size of buffer is not a multiple of the data size");
                }
                let data_len = len / (*n_copies as usize);
                buffer.split_at_mut(data_len)
            }
            Policy::ReedSolomon(n_ecc) => {
                if len <= (*n_ecc as usize) {
                    panic!("Reed-Solomon: The number of data bits plus the amount of error correction bits is too small");
                }
                let data_len = len - (*n_ecc as usize);
                buffer.split_at_mut(data_len)
            }
            _ => buffer.split_at_mut(buffer.len() - 1),
        }
    }

    fn split_buffer<'a>(&self, buffer: &'a [u8]) -> (&'a [u8], &'a [u8]) {
        let len = buffer.len();
        match self {
            Policy::Redundancy(n_copies) => {
                if len % (*n_copies as usize) != 0 {
                    panic!("Redundancy: Size of buffer is not a multiple of the data size");
                }
                let data_len = len / (*n_copies as usize);
                buffer.split_at(data_len)
            }
            Policy::ReedSolomon(n_ecc) => {
                if len <= (*n_ecc as usize) {
                    panic!("Reed-Solomon: The number of data bits plus the amount of error correction bits is too small");
                }
                let data_len = len - (*n_ecc as usize);
                buffer.split_at(data_len)
            }
            _ => buffer.split_at(buffer.len() - 1),
        }
    }

    /// Determines if the slice si corrupted give the current policy
    fn is_corrupted(&self, buffer: &[u8]) -> bool {
        let (data, ecc) = self.split_buffer(buffer);

        match self {
            Policy::Redundancy(n_copies) => {
                let data_len = data.len();
                for byte in 0..data_len {
                    let val = buffer[byte];
                    for copy in 1..*n_copies {
                        if val != buffer[(copy as usize) * data_len + byte] {
                            return true;
                        }
                    }
                }
                false
            }
            Policy::ReedSolomon(n_ecc) => {
                let dec = Decoder::new(*n_ecc as usize);
                dec.is_corrupted(buffer)
            }
            _ => false,
        }
    }

    /// If any errors are present in the buffer, this wlil correct them
    fn correct_buffer(&self, buffer: &mut [u8]) -> u32 {
        match self {
            Policy::Redundancy(n_copies) => {
                let (data, _) = self.split_buffer(buffer);
                let n_copies = *n_copies as usize;
                (0..data.len()).map(|index| correct_bits_redundant(buffer, n_copies, index)).sum()
            }
            Policy::ReedSolomon(correction_bits) => {
                let dec = Decoder::new(*correction_bits as usize);
                let (corrected, n_errors) = dec.correct_err_count(buffer, None).unwrap();
                let (data, ecc) = self.split_buffer_mut(buffer);
                data.clone_from_slice(corrected.data());
                ecc.clone_from_slice(corrected.ecc());
                n_errors as u32
            }
            _ => 0,
        }
    }
}

/// Metadata that is adjacent to the actual data stored.
#[repr(C)]
#[derive(Clone)]
pub struct AllocBlock {
    /// Policies to be applied to the data in order from 0 to MAX_POLICIES
    policies: [Policy; MAX_POLICIES],

    // The full size of the allocated block
    full_size: usize,

    // The amount of the data allocated
    length: usize,
}
/*
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
*/
// #[cfg(light_weight)]
impl AllocBlock {
    fn ptr(&self) -> *mut u8 {
        let block_ptr = self as *const AllocBlock;
        unsafe {
            let block_ptr: *mut u8 = block_ptr as *mut u8;
            block_ptr.add(std::mem::size_of::<AllocBlock>())
        }
    }

    fn alloced_region(&self) -> *mut u8 {
        self as *const AllocBlock as *mut u8
    }

    pub fn get_block<'a>(ptr: *const u8) -> &'a AllocBlock {
        unsafe {
            (ptr.sub(std::mem::size_of::<AllocBlock>()) as *const AllocBlock)
                .as_ref()
                .expect("Null ptr")
        }
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr() as *mut c_void
    }

    /// TODO: Convert to tree
    fn correct_buffer(&mut self) -> u32 {
        let pol = self.policies.clone();
        // can't pass &mut self to enforce_policy while also iterating over original policy array
        pol.iter().map(|i| i.correct_buffer(unsafe{self.full_slice()})).sum()
    }

    fn is_corrupted(&self) -> bool {
        panic!("Not Implemented")
    }

    /// TODO: Convert to tree
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

    pub fn new<'a>(size: usize, policies: &[Policy; MAX_POLICIES], zeroed: bool) -> &'a mut AllocBlock {
        let full_size: usize = AllocBlock::size_of(size, policies);
        let res = Layout::from_size_align(full_size + std::mem::size_of::<AllocBlock>(), 16);

        match res {
            Ok(_val) => {
                let layout = res.unwrap();

                let block_ptr: *mut u8 = if zeroed {
                    unsafe { alloc_zeroed(layout) }
                } else  {
                    unsafe { alloc(layout) }
                };
                let block: &'a mut AllocBlock;

                unsafe {
                    block = std::mem::transmute(block_ptr);
                }

                // TODO: Initialize block here
                block.full_size = full_size;
                block.length = size;
                block.policies = *policies;
                block
            }
            Err(_e) => panic!("Invalid layout arguments"),
        }
    }

    pub fn realloc<'a>(&self, size: usize, policies: &[Policy; MAX_POLICIES]) -> &'a mut AllocBlock {
        let full_size: usize = AllocBlock::size_of(size, policies);
        let res = Layout::from_size_align(full_size + std::mem::size_of::<AllocBlock>(), 16);

        match res {
            Ok(_val) => {
                let layout = res.unwrap();

                let old_block = (*self).clone();

                let block_ptr: *mut u8 = unsafe { realloc(self.alloced_region(), layout, size) };

                let block: &'a mut AllocBlock;

                unsafe { block = std::mem::transmute(block_ptr); }

                block.full_size = full_size;
                block.length    = size;
                block.policies  = old_block.policies;
                block
            },
            Err(_e) => panic!("Invalid layout arguments")
        }
    }

    pub fn free_ptr(ptr: *const u8) {
        let block_ref = AllocBlock::get_block(ptr);
        let res = Layout::from_size_align(block_ref.full_size + std::mem::size_of::<AllocBlock>(), 16).expect("layout failed");
        let ptr = block_ref.alloced_region();
        unsafe { dealloc(ptr, res); }
    }

    unsafe fn full_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr(), self.full_size) }
    }

    fn data_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr(), self.length) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redundancy_check() {
        let r_policy = Policy::Redundancy(3);

        let block: &mut AllocBlock = AllocBlock::new(1, &[r_policy, Policy::Nil, Policy::Nil]);

        // Create errors
        // unsafe {
        //     *block.ptr.add(0) = 0b1111;
        //     *block.ptr.add(1) = 0b1010;
        //     *block.ptr.add(2) = 0b0000;
        // }
        let slice = unsafe { block.full_slice() };

        slice[0] = 0b1111;
        slice[1] = 0b1010;
        slice[2] = 0b0000;

        // assert_eq!(r_policy.enforce_redundancy(block, 0), 4);

        for idx in 0..3 {
            unsafe {
                // assert_eq!(*block.ptr.add(idx), 0b1010 as u8);
            }
        }
    }
}
