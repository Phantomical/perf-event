//! Perf events for Intel x86 and x86-64 CPUs.

mod msr;

pub use self::msr::{Msr, MsrId};
