// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.

pub use bitfield_impl::bitfield;
pub use bitfield_impl::specifier;
pub use bitfield_impl::BitfieldSpecifier;

pub mod access;
pub use access::*;
pub mod check;

// TODO other things

pub trait Specifier {
    const BITS: usize;
    type T;  // Refactor for test-06, the inner value type
    type V;  // Refactor for test-06, the value type of get/set interface
}

// Define bits specifier.
// This function like macro will generate a sort of specifer struct for 1 to n.
// Concretely, call specifier!(64) will expand codes below:
// pub enum B1 {}
// impl Specifier for B1 { const BITS: u8 = 1; }
//     .    .    .
//     .    .    .
//     .    .    .
// pub enum B64 {}
// impl Specifier for B64 { const BITS: u8 = 64; }
specifier!(64);

// Implement Specifier for bool.
impl Specifier for bool {
    const BITS: usize = 1;
    type T = u8;
    type V = bool;
}

// For test-06.
// Our implementation of get/set method has a conversion between Specifier::T & Specifier::V.
// For compatiable with the `bool` type, here needs a trait behave as the ::core::convert::Into,
// because we can't impl Into<bool> for u8.
pub trait BInto<T> {
    fn binto(self) -> T;
}

impl<T> BInto<T> for T {
    fn binto(self) -> T {
        self
    }
}

impl BInto<bool> for u8 {
    fn binto(self) -> bool {
        match self {
            0 => false,
            _ => true,
        }
    }
}

impl BInto<u8> for bool {
    fn binto(self) -> u8 {
        match self {
            false => 0,
            true => 1,
        }
    }
}
