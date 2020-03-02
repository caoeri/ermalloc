extern crate alloc;

use alloc::alloc::{alloc, alloc_zeroed, dealloc, realloc, Layout};
use core::convert::TryFrom;
use core::iter::Iterator;
use core::mem::transmute;

use crate::weak::*;

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
// Support Create (Done), Read (By default), Update (By default), Delete (Done)
// Full alloc (Done), realloc, dealloc (Done)
// Proper warnings for poor allocations
// Cleaner interface for size propagation upwards (All hidden!)
// Interface: is_corrupted (Done), apply (Done), correct (Done)

// Testing

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

    // Count bits
    let mut corrected: u8 = 0;
    for bit in 0..8 {
        let mask = 1 << bit;
        let mut count: [u32; 2] = [0, 0];

        (0..n_copies).map(|i| buffer[i * data_len + index]).for_each(|byte| {
            count[((byte & mask) >> bit) as usize] += 1;
        });

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
                (0..data.len())
                    .map(|index| correct_bits_redundant(buffer, n_copies, index))
                    .sum()
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

    fn apply_policy(&self, buffer: &mut [u8]) {
        match self {
            Policy::Redundancy(n_copies) => {
                if buffer.len() % (*n_copies as usize) != 0 {
                    panic!("Redundancy: Size of buffer is not a multiple of the data size");
                }
                let data_len = buffer.len() / (*n_copies as usize);
                let (data, err) = self.split_buffer_mut(buffer);
                for slice in err.chunks_exact_mut(data_len).skip(1) {
                    slice.copy_from_slice(data)
                }
            }
            Policy::ReedSolomon(correction_bits) => {
                let enc = Encoder::new(*correction_bits as usize);
                let (data, err) = self.split_buffer_mut(buffer);
                let encoded = enc.encode(data);
                err.copy_from_slice(encoded.ecc());
            }
            _ => (),
        }
    }

    /// Unwraps one layer of policy application to get the data
    fn get_data_mut<'a>(&self, buffer: &'a mut [u8]) -> &'a mut [u8] {
        let (data, _) = self.split_buffer_mut(buffer);
        data
    }

    fn get_data<'a>(&self, buffer: &'a [u8]) -> &'a [u8] {
        let (data, _) = self.split_buffer(buffer);
        data
    }
}

/// Metadata that is adjacent to the actual data stored.
#[repr(C)]
pub struct AllocBlock {
    /// Policies to be applied to the data in order from 0 to MAX_POLICIES
    policies: [Policy; MAX_POLICIES],

    // The full size of the allocated block
    full_size: usize,

    // The amount of the data allocated
    length: usize,

    // A WeakMut holds a references
    // We can figure out how we want to manage this thing later
    weak_exists: bool,
}

impl Weakable for AllocBlock {
    fn weak_exists(&self) -> bool {
        self.weak_exists
    }

    fn set_weak_exists(&mut self) {
        self.weak_exists = true;
    }

    fn reset_weak_exists(&mut self) {
        self.weak_exists = false;
    }
}

// #[cfg(light_weight)]
impl AllocBlock {
    pub fn ptr_ffi<'a>(mut w: Weak<'a, AllocBlock>) -> *mut u8 {
        w.get_ref().expect("ptr_ffi").ptr()
    }

    fn ptr(&self) -> *mut u8 {
        let block_ptr = self as *const AllocBlock;
        unsafe {
            let block_ptr: *mut u8 = block_ptr as *mut u8;
            block_ptr.add(core::mem::size_of::<AllocBlock>())
        }
    }

    fn size_of(desired_size: usize, policies: &[Policy; MAX_POLICIES]) -> usize {
        let mut full_size = desired_size;
        for p in policies.iter().rev() {
            match p {
                Policy::Redundancy(num_copies) => {
                    full_size *= usize::try_from(*num_copies).unwrap()
                }
                Policy::ReedSolomon(n_ecc) => full_size += usize::try_from(*n_ecc).unwrap(),
                _ => (),
            }
        }
        full_size
    }

    pub fn new<'a>(
        size: usize,
        policies: &[Policy; MAX_POLICIES],
        zeroed: bool,
    ) -> WeakMut<'a, AllocBlock> {
        let full_size: usize = AllocBlock::size_of(size, policies);
        let res = Layout::from_size_align(full_size + core::mem::size_of::<AllocBlock>(), 16);

        match res {
            Ok(layout) => {
                let block_ptr: *mut u8 = unsafe {
                    if zeroed {
                        alloc_zeroed(layout)
                    } else {
                        alloc(layout)
                    }
                };
                let block: &'a mut AllocBlock;

                block = unsafe { &mut *(block_ptr as *mut AllocBlock) };
                block.full_size = full_size;
                block.length = size;
                block.policies = *policies;
                block.weak_exists = false;

                if zeroed {
                    block.apply_policy();
                }
                WeakMut::from(block)
            }
            Err(_e) => panic!("Invalid layout arguments"),
        }
    }

    pub fn renew<'a>(
        mut w: WeakMut<'a, AllocBlock>,
        new_size: usize,
        new_policies: &[Policy; MAX_POLICIES],
    ) -> WeakMut<'a, AllocBlock> {
        let new_full_size = AllocBlock::size_of(new_size, new_policies);
        let new_res =
            Layout::from_size_align(new_full_size + core::mem::size_of::<AllocBlock>(), 16);

        match new_res {
            Ok(layout) => {
                let new_block_ptr = unsafe { realloc(w.as_ptr() as *mut u8, layout, new_size) };

                let new_block: &'a mut AllocBlock;

                new_block = unsafe { &mut *(new_block_ptr as *mut AllocBlock) };
                new_block.full_size = new_full_size;
                new_block.length = new_size;
                new_block.policies = *new_policies;
                new_block.weak_exists = false;
                new_block.apply_policy();
                WeakMut::from(new_block)
            }
            Err(_e) => panic!("Invalid layout arguments"),
        }
    }

    pub fn from_usr_ptr<'a>(ptr: *const u8) -> Weak<'a, AllocBlock> {
        let block = unsafe { &*(ptr as *const AllocBlock).sub(1) };
        Weak::from(block)
    }

    pub fn from_usr_ptr_mut<'a>(ptr: *mut u8) -> WeakMut<'a, AllocBlock> {
        let block = unsafe { &mut *(ptr as *mut AllocBlock).sub(1) };
        WeakMut::from(block)
    }

    pub fn drop<'a>(mut w: WeakMut<'a, AllocBlock>) {
        w.get_ref_mut()
            .expect("Called drop on invalid WeakMut")
            .drop_ref();
    }

    fn drop_ref(&mut self) {
        let full_size: usize = AllocBlock::size_of(self.length, &self.policies);
        let res = Layout::from_size_align(full_size + core::mem::size_of::<AllocBlock>(), 16);

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

    fn full_slice_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr(), self.full_size) }
    }

    fn full_slice(&self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr(), self.full_size) }
    }

    pub fn data_slice_ffi<'a>(w: WeakMut<'a, AllocBlock>) -> &mut [u8] {
        w.get_ref_mut()
            .expect("data_slice_ffi")
            .full_slice()
    }

    fn data_slice(&self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr(), self.length) }
    }
    
    pub fn correct_buffer_ffi<'a>(w: WeakMut<'a, AllocBlock>) -> u32 {
        w.get_ref_mut()
            .expect("correct_buffer_ffi")
            .correct_buffer()
    }

    fn correct_buffer(&mut self) -> u32 {
        let buffer = unsafe { self.full_slice() };
        self.correct_bits_helper(0, buffer)
    }

    fn correct_bits_helper(&self, index: usize, full_buffer: &mut [u8]) -> u32 {
        let corrected_bits = match index == MAX_POLICIES {
            true => return 0,
            false => match self.policies[index] {
                Policy::Nil => return 0,
                Policy::Redundancy(n_copies) => {
                    if full_buffer.len() % (n_copies as usize) != 0 {
                        panic!("Redundancy: Size of buffer is not a multiple of the data size");
                    }
                    let data_len = full_buffer.len() / (n_copies as usize);

                    full_buffer
                        .chunks_exact_mut(data_len)
                        .map(|slice| self.correct_bits_helper(index + 1, slice))
                        .sum()
                }
                _ => self
                    .correct_bits_helper(index + 1, self.policies[index].get_data_mut(full_buffer)),
            },
        };

        corrected_bits + self.policies[index].correct_buffer(full_buffer)
    }

    fn is_corrupted(&self) -> bool {
        let buffer = unsafe { self.full_slice() };
        self.is_corrupted_helper(0, buffer)
    }

    fn is_corrupted_helper(&self, index: usize, full_buffer: &[u8]) -> bool {
        let corrected_bits = match index == MAX_POLICIES {
            true => return false,
            false => match self.policies[index] {
                Policy::Nil => return false,
                _ => {
                    self.is_corrupted_helper(index + 1, self.policies[index].get_data(full_buffer))
                }
            },
        };

        corrected_bits || self.policies[index].is_corrupted(full_buffer)
    }

    pub fn apply_policy_ffi<'a>(w: WeakMut<'a, AllocBlock>) {
        w.downgrade()
            .get_ref()
            .expect("apply policy ffi")
            .apply_policy();
    }

    fn apply_policy(&self) {
        let buffer = unsafe { self.full_slice() };
        self.apply_policy_helper(0, buffer)
    }

    fn apply_policy_helper(&self, index: usize, full_buffer: &mut [u8]) {
        match index == MAX_POLICIES {
            true => return,
            false => match self.policies[index] {
                Policy::Nil => return,
                _ => self
                    .apply_policy_helper(index + 1, self.policies[index].get_data_mut(full_buffer)),
            },
        };

        self.policies[index].apply_policy(full_buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redundancy_check() {
        let block = AllocBlock::new(1, &[Policy::Redundancy(3), Policy::Nil, Policy::Nil], false);

        // Create errors
        // unsafe {
        //     *block.ptr.add(0) = 0b1111;
        //     *block.ptr.add(1) = 0b1010;
        //     *block.ptr.add(2) = 0b0000;
        // }
        let block_ref = block.get_ref_mut().unwrap();
        let slice = unsafe { block_ref.full_slice() };
        slice[0] = 0b1111;
        slice[1] = 0b1010;
        slice[2] = 0b0000;
        assert_eq!(block_ref.is_corrupted(), true);
        assert_eq!(block_ref.correct_buffer(), 4);
        assert_eq!(block_ref.is_corrupted(), false);
        let slice = unsafe { block_ref.full_slice() };
        for idx in 0..3 {
            unsafe {
                assert_eq!(slice[idx], 0b1010 as u8);
            }
        }
    }

    #[test]
    fn fec_check() {
        let block = AllocBlock::new(
            1,
            &[Policy::ReedSolomon(3), Policy::Nil, Policy::Nil],
            false,
        );

        let block_ref = block.get_ref_mut().unwrap();
        let slice = unsafe { block_ref.full_slice() };
        slice[0] = 0b1111;
        block_ref.apply_policy();
        let slice = unsafe { block_ref.full_slice() };
        slice[0] = 0b1011;
        assert_eq!(block_ref.is_corrupted(), true);
        assert_eq!(block_ref.correct_buffer(), 1);
        assert_eq!(block_ref.is_corrupted(), false);
        let slice = unsafe { block_ref.full_slice() };
        assert_eq!(slice[0], 0b1111 as u8);
    }
}
