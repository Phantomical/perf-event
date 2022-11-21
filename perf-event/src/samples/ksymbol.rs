use std::ffi::OsString;
use std::os::unix::prelude::OsStringExt;

use bitflags::bitflags;
use perf_event_open_sys::bindings;

use super::{Parse, ParseBuf, RecordEvent};

/// KSYMBOL records indicate symbols being registered or unregistered within
/// the kernel.
///
/// This struct corresponds to `PERF_RECORD_KSYMBOL`. See the [manpage] for
/// more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct KSymbol {
    pub addr: u64,
    pub len: u32,
    pub ksym_type: KSymbolType,
    pub flags: KSymbolFlags,
    pub name: OsString,
}

enum_binding! {
    /// The type of the kernel symbol.
    pub struct KSymbolType : u16 {
        const UNKNOWN = bindings::PERF_RECORD_KSYMBOL_TYPE_UNKNOWN as _;
        const BPF = bindings::PERF_RECORD_KSYMBOL_TYPE_BPF as _;
        const OOL = bindings::PERF_RECORD_KSYMBOL_TYPE_OOL as _;
    }
}

bitflags! {
    /// Flags for [`KSymbol`].
    pub struct KSymbolFlags : u16 {
        /// If set, this means that the symbol is being unregistered.
        const UNREGISTER = bindings::PERF_RECORD_KSYMBOL_FLAGS_UNREGISTER as _;
    }
}

impl KSymbolFlags {
    /// Create a new set of flags from the underlying bits.
    pub fn new(bits: u16) -> Self {
        Self { bits }
    }
}

impl Parse for KSymbol {
    fn parse<B: bytes::Buf>(_: &super::ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            addr: buf.get_u64_ne(),
            len: buf.get_u32_ne(),
            ksym_type: KSymbolType(buf.get_u16_ne()),
            flags: KSymbolFlags::new(buf.get_u16_ne()),
            name: {
                let mut vec = buf.parse_remainder();

                // Remove padding nul bytes from the entry
                while let Some(b'\0') = vec.last() {
                    vec.pop();
                }

                OsString::from_vec(vec)
            },
        }
    }
}

impl From<KSymbol> for RecordEvent {
    fn from(ksymbol: KSymbol) -> Self {
        Self::KSymbol(ksymbol)
    }
}
