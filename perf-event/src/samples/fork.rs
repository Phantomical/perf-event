use bytes::Buf;

use super::{Parse, ParseConfig, RecordEvent};

/// FORK records indicate that a process called [`fork(2)`] successfully.
///
/// This struct corresponds to `PERF_RECORD_FORK`. See the [manpage] for more
/// documentation.
///
/// [`fork(2)`]: https://man7.org/linux/man-pages/man2/fork.2.html
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct Fork {
    pub pid: u32,
    pub ppid: u32,
    pub tid: u32,
    pub ptid: u32,
    pub time: u64,
}

impl Parse for Fork {
    fn parse<B: Buf>(_: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            pid: buf.get_u32_ne(),
            ppid: buf.get_u32_ne(),
            tid: buf.get_u32_ne(),
            ptid: buf.get_u32_ne(),
            time: buf.get_u64_ne(),
        }
    }
}

impl From<Fork> for RecordEvent {
    fn from(fork: Fork) -> Self {
        RecordEvent::Fork(fork)
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
            0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
        ];

        let exit = Fork::parse(&ParseConfig::default(), &mut bytes);

        assert_eq!(exit.pid, 0x1010);
        assert_eq!(exit.ppid, 0x0500);
        assert_eq!(exit.tid, 0x01);
        assert_eq!(exit.ptid, 0x02);
        assert_eq!(exit.time, 0x0400000003);
    }
}
