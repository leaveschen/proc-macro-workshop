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

pub mod access;
pub use access::*;

// TODO other things

pub trait Specifier {
    const BITS: usize;
    type T;
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



