use super::{Parse, ParseBuf, RecordEvent};

/// TEXT_POKE records indicate a change in the kernel text.
///
/// This struct corresponds to `PERF_RECORD_TEXT_POKE`. See the [manpage] for
/// more documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct TextPoke {
    /// The address of the change.
    pub addr: u64,

    /// The old bytes at `addr`.
    pub old_bytes: Vec<u8>,

    /// The new bytes at `addr`.
    pub new_bytes: Vec<u8>,
}

impl Parse for TextPoke {
    fn parse<B: bytes::Buf>(_: &super::ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        let addr = buf.get_u64_ne();
        let old_len = buf.get_u16_ne() as usize;
        let new_len = buf.get_u16_ne() as usize;

        Self {
            addr,
            old_bytes: buf.parse_vec(old_len),
            new_bytes: buf.parse_vec(new_len),
        }
    }
}

impl From<TextPoke> for RecordEvent {
    fn from(evt: TextPoke) -> Self {
        Self::TextPoke(evt)
    }
}
