#![no_std]
#![cfg_attr(test, no_main)]

use my_app as _; // memory layout + panic handler

#[defmt_test::tests]
mod tests {}
