use perf_event_open_sys::bindings;

use crate::events::Event;

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
/// ```no_run
/// use perf_event::events::Raw;
/// use perf_event::{Builder, Group};
///
/// // Raw config values for an ARMv8 PMU.
/// let INSNS_RETIRED: Raw = Raw::new(0x08);
/// let CPU_CYCLES: Raw = Raw::new(0x11);
///
/// let mut group = Group::new()?;
/// let raw_insns_retired = group.add(&Builder::new(INSNS_RETIRED).include_kernel())?;
/// let raw_cpu_cycles = group.add(&Builder::new(CPU_CYCLES).include_kernel())?;
/// # std::io::Result::Ok(())
/// ```
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Raw {
    /// Raw config of the event
    pub config: u64,

    /// Raw config1 of the event
    pub config1: u64,

    /// Raw config2 of the event
    pub config2: u64,
}

impl Raw {
    /// Create a new raw event value with the given config value.
    ///
    /// This sets all other config fields to zero. For most events this should
    /// be sufficient but in other cases methods are provided to set those
    /// fields as well.
    pub const fn new(config: u64) -> Self {
        Raw {
            config,
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
        attr.config1 = self.config1;
        attr.config2 = self.config2;
    }
}
