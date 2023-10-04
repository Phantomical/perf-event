use crate::events::Event;
use perf_event_open_sys::bindings;

/// A Raw event
///
/// A Raw event can be specified with:
///
/// - config, config1 and config2
///
/// Example:
///
/// ```
/// # use perf_event::events::Raw;
/// # use perf_event::{Builder, Group};
/// # fn main() -> std::io::Result<()> {
/// let INSNS_RETIRED: Raw = Raw::new().config(0x08);
/// let CPU_CYCLES: Raw = Raw::new().config(0x11);
///
/// let mut group = Group::new()?;
/// let raw_insns_retired = group.add(&Builder::new(INSNS_RETIRED).include_kernel())?;
/// let raw_cpu_cycles = group.add(&Builder::new(CPU_CYCLES).include_kernel())?;
/// # Ok(())
/// }
/// ```
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Raw {
    /// Raw config of the event
    pub config: u64,

    /// Raw config1 of the event
    pub config1: u64,

    /// Raw config2 of the event
    pub config2: u64,
}

impl Raw {
    /// Create a new Raw event
    pub fn new() -> Self {
        Raw {
            config: 0,
            config1: 0,
            config2: 0,
        }
    }

    /// Set config
    pub fn config(mut self, config: u64) -> Self {
        self.config = config;
        self
    }

    /// Set config1
    pub fn config1(mut self, config1: u64) -> Self {
        self.config1 = config1;
        self
    }

    /// Set config2
    pub fn config2(mut self, config2: u64) -> Self {
        self.config2 = config2;
        self
    }
}

impl Event for Raw {
    fn update_attrs(self, attr: &mut bindings::perf_event_attr) {
        attr.type_ = bindings::PERF_TYPE_RAW;
        attr.config = self.config;
        attr.__bindgen_anon_3.config1 = self.config1;
        attr.__bindgen_anon_4.config2 = self.config2;
    }
}
