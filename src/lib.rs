#![no_std]
#![feature(alloc_error_handler)]
#![feature(lang_items)]
#![allow(dead_code)]

mod policies;
mod weak;
mod ffi;
mod alloc;
mod panic;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
