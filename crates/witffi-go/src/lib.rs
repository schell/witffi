//! Go bindings code generator for WIT interfaces.
//!
//! Generates a single `.go` file that calls the existing C-ABI functions via
//! CGo. The generated Go code deep-copies all data into native Go types and
//! frees the C memory immediately, matching Go's garbage-collected memory model.

pub mod generate;

pub use generate::GoGenerator;
