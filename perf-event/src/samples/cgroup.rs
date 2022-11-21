use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

use super::{Parse, ParseBuf, RecordEvent};

/// CGROUP records indicate when a new cgroup is created and activated.
///
/// This struct corresponds to `PERF_RECORD_CGROUP`. See the [manpage] for more
/// documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct Cgroup {
    /// The cgroup ID.
    pub id: u64,

    /// Path of the cgroup from the root.
    pub path: OsString,
}

impl Parse for Cgroup {
    fn parse<B: bytes::Buf>(_: &super::ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            id: buf.get_u64_ne(),
            path: {
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

impl From<Cgroup> for RecordEvent {
    fn from(cgroup: Cgroup) -> Self {
        Self::Cgroup(cgroup)
    }
}
