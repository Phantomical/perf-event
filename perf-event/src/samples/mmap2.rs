use bytes::Buf;
use std::ffi::OsString;
use std::os::unix::prelude::OsStringExt;

use super::{Parse, ParseBuf, ParseConfig, RecordEvent};

/// MMAP2 events record memory mappings with extra info compared to MMAP
/// records.
///
/// This struct corresponds to `PERF_RECORD_MMAP2`. See the [manpage] for more
/// documentation here.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct Mmap2 {
    pub pid: u32,
    pub tid: u32,
    pub addr: u64,
    pub len: u64,
    pub pgoff: u64,
    pub maj: u32,
    pub min: u32,
    pub ino: u64,
    pub ino_generation: u64,
    pub prot: u32,
    pub flags: u32,
    pub filename: OsString,
}

impl Parse for Mmap2 {
    fn parse<B: Buf>(_: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            pid: buf.get_u32_ne(),
            tid: buf.get_u32_ne(),
            addr: buf.get_u64_ne(),
            len: buf.get_u64_ne(),
            pgoff: buf.get_u64_ne(),
            maj: buf.get_u32_ne(),
            min: buf.get_u32_ne(),
            ino: buf.get_u64_ne(),
            ino_generation: buf.get_u64_ne(),
            prot: buf.get_u32_ne(),
            flags: buf.get_u32_ne(),
            filename: {
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

impl From<Mmap2> for RecordEvent {
    fn from(mmap: Mmap2) -> Self {
        RecordEvent::Mmap2(mmap)
    }
}
