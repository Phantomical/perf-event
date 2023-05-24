use crate::events::Event;
use c_enum::c_enum;
use perf_event_open_sys::bindings;
use std::io;
use std::sync::atomic::{AtomicU32, Ordering};

// 0 will never be the PMU value for msr.
// We use it as a flag value to indicate that this has not been initialized.
static MSR_TYPE: AtomicU32 = AtomicU32::new(0);


c_enum! {
    /// MSR events on all x86 CPUs
    #[derive(Clone, Copy, Eq, PartialEq, Hash)]
    pub enum MSRConfig: u64 {
        /// Time Stamp Counter
        /// TSC increases at the CPU base frequency.
        TSC = 0x0,
        /// Actual Performance Frequency Clock Count
        APERF = 0x1,
        /// Maximum Performance Frequency Clock Count
        /// (APERF / MPERF) * CPU_BASE_FREQUENCY is the running CPU frequency
        MPERF = 0x2,
    }
}

/// MSR event
/// 
/// MSR event allow you to read the per-CPU x86 MSR counters. The MSR event might not exist on non-x86 CPUs
/// and the type of the pmu is dynamic, so MSREvent::with_config searches /sys/fs/event_source/devices/msr/type
/// checking whether it exists and fetching the run-time type value.
pub struct MSREvent {
    ty: u32,
    config: MSRConfig,
}

impl MSREvent {
    /// Create a MSR event.
    ///
    /// # Errors
    /// This will attempt to read the PMU type from
    /// `/sys/bus/event_source`. It will return an error if the MSR PMU is missing.
    /// Please notice that because MSR events don't support user-only counting, please clear the kernel and
    /// hv exclusive bits by calling exclude_hv(false) exclude_kernel(false)
    pub fn with_config(config: MSRConfig) -> io::Result<Self> {
        match MSR_TYPE.load(Ordering::Relaxed) {
            0 => {
                let text = std::fs::read_to_string("/sys/bus/event_source/devices/msr/type")?;
                let ty = text
                    .trim_end()
                    .parse()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                MSR_TYPE.store(ty, Ordering::Relaxed);
                return Ok(Self { ty, config });
            }
            ty => {
                return Ok(Self { ty, config });
            }
        };
    }
}

impl Event for MSREvent {
    fn update_attrs(self, attr: &mut bindings::perf_event_attr) {
        attr.type_ = self.ty;
        attr.config = self.config.into();
    }
}
