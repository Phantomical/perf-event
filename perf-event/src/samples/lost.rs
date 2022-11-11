use bytes::Buf;

use super::{Parse, ParseConfig, RecordEvent};

/// Lost records indicate when events are dropped by the kernel.
///
/// This will happen when the sampler ring buffer fills up and there is no
/// space left for events to be inserted.
#[derive(Copy, Clone, Debug)]
pub struct Lost {
    /// The unique event ID for the samples that were lost.
    pub id: u64,

    /// The number of events that were lost.
    pub lost: u64,
}

impl Parse for Lost {
    fn parse<B: Buf>(_: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            id: buf.get_u64_ne(),
            lost: buf.get_u64_ne(),
        }
    }
}

impl From<Lost> for RecordEvent {
    fn from(lost: Lost) -> Self {
        RecordEvent::Lost(lost)
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
            0x10, 0x00, 0x99, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xAF, 0x00, 0x00, 0x00, 0x7B, 0x00, 0x00
        ];

        let lost = Lost::parse(&ParseConfig::default(), &mut bytes);

        assert_eq!(lost.id, 0x990010);
        assert_eq!(lost.lost, 0x7B000000AF00);
    }
}
