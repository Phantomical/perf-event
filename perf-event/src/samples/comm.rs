use std::ffi::OsString;
use std::os::unix::prelude::OsStringExt;

use bytes::Buf;

use super::{Parse, ParseBuf, ParseConfig, RecordEvent};

/// COMM records indicate changes in process names.
///
/// There are multiple ways that this could happen: [`execve(2)`],
/// [`prctl(PR_SET_NAME)`], as well as writing to `/proc/self/comm`.
///
/// Since Linux 3.10 the kernel will set the [`COMM_EXEC`] bit in
/// [`Record::misc`] if the record is due to an [`execve(2)`] syscall.
/// You can use [`Builder::comm_exec`] to detect whether this is supported.
///
/// This struct corresponds to `PERF_RECORD_COMM`. See the [manpage] for more
/// documentation.
///
/// [`execve(2)`]: https://man7.org/linux/man-pages/man2/execve.2.html
/// [`prctl(PR_SET_NAME)`]: https://man7.org/linux/man-pages/man2/prctl.2.html
/// [`COMM_EXEC`]: crate::samples::RecordMiscFlags::COMM_EXEC
/// [`Record::misc`]: crate::samples::Record::misc
/// [`Builder::comm_exec`]: crate::Builder::comm_exec
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct Comm {
    pub pid: u32,
    pub tid: u32,
    pub comm: OsString,
}

impl Parse for Comm {
    fn parse<B: Buf>(_: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            pid: buf.get_u32_ne(),
            tid: buf.get_u32_ne(),
            comm: {
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

impl From<Comm> for RecordEvent {
    fn from(comm: Comm) -> Self {
        RecordEvent::Comm(comm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(not(target_endian = "little"), ignore)]
    fn test_parse() {
        #[rustfmt::skip]
        let mut bytes: &[u8] = &[
            0x10, 0x10, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00,
            b't', b'e', b's', b't', 0x00, 0x00, 0x00, 0x00
        ];

        let comm = Comm::parse(&ParseConfig::default(), &mut bytes);

        assert_eq!(comm.pid, 0x1010);
        assert_eq!(comm.tid, 0x0500);
        assert_eq!(comm.comm, "test");
    }
}
