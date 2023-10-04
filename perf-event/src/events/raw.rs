use crate::events::Event;
use perf_event_open_sys::bindings;

/// A raw perf counter for the current CPU.
/// 
/// Most CPUs have additional counters beyond those provided by the kernel.
/// `Raw` events allow you to access those events. Note that the values needed
/// to configure raw events a liable to change between CPU vendors and even
/// different hardware revisions of the same platform.
/// 
/// The event can be chosen by setting the `config` field. Most events will
/// only need that, but others may require setting the `config1` or `config2`
/// fields as well.
/// 
/// To find the config values required for counters consult your CPU manual.
/// - For Intel CPUs, see the Intel Software Developer Manual, volume 3B.
/// - For AMD, see the AMD BIOS and Kernel Developer Guide.
/// - Other vendors should have equivalent documentation.
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
    /// Create a new raw event.
    /// 
    /// The event has all fields zeroed out and will likely need to be configured
    /// further to get the counter configuration you want.
    pub const fn new() -> Self {
        Raw {
            config: 0,
            config1: 0,
            config2: 0,
        }
    }

    /// Set config
    pub const fn config(mut self, config: u64) -> Self {
        self.config = config;
        self
    }

    /// Set config1
    pub const fn config1(mut self, config1: u64) -> Self {
        self.config1 = config1;
        self
    }

    /// Set config2
    pub const fn config2(mut self, config2: u64) -> Self {
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
