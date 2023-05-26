use crate::events::Event;
use c_enum::c_enum;
use perf_event_open_sys::bindings;
use std::io;
use std::sync::atomic::{AtomicU32, Ordering};

// 0 will never be the PMU value for msr.
// We use it as a flag value to indicate that this has not been initialized.
static MSR_TYPE: AtomicU32 = AtomicU32::new(0);

c_enum! {
    /// The [MSRs] supported by the [Linux msr pmu]
    ///
    /// [Linux msr pmu]: https://github.com/torvalds/linux/blob/master/arch/x86/events/msr.c
    /// [MSRs]: https://github.com/torvalds/linux/blob/master/arch/x86/include/asm/msr-index.h
    #[derive(Clone, Copy, Eq, PartialEq, Hash)]
    pub enum MsrId: u64 {
        /// x86 Time Stamp Counter (MSR_IA32_TSC).
        TSC = 0x0,
        /// x86 Actual Performance Frequency Clock (MSR_IA32_APERF).
        APERF = 0x1,
        /// x86 Maximum Performance Frequency Clock Count (MSR_IA32_MPERF)
        ///
        /// (APERF / MPERF) * CPU base frequency = running CPU frequency
        MPERF = 0x2,
        /// Intel The Productive Performance MSR (MSR_PPERF).
        ///
        /// PPERF is similar to APERF but only increased for non-halted cycles.
        PPERF = 0x3,
        /// Intel System Management Interrupt Counter (MSR_SMI_COUNT).
        SMI = 0x4,
        /// AMD Performance Timestamp Counter (MSR_F15H_PTSC).
        PTSC = 0x5,
        /// AMD Instructions Retired Performance Counter (MSR_F17H_IRPERF)
        IRPERF = 0x6,
        /// Intel Thermal Status MSR (MSR_IA32_THERM_STATUS).
        THERM = 0x7,
    }
}

/// The MSR event allowing you to use the MSRs defined in the [Linux msr pmu].
///
/// [Linux msr pmu]: https://github.com/torvalds/linux/blob/master/arch/x86/events/msr.c
pub struct Msr {
    ty: u32,
    config: MsrId,
}

impl Msr {
    /// Create a MSR event.
    ///
    /// Please notice that because MSR events don't support user-only counting,
    /// please clear the kernel and hv exclusive bits by calling
    /// [exclude_kernel](crate::Builder::exclude_hv)(`false`) and
    /// [exclude_kernel](crate::Builder::exclude_kernel)(`false`).
    ///
    /// # Errors
    /// This will attempt to read the PMU type from
    /// `/sys/bus/event_source`. It will return an error if the MSR PMU is
    /// missing.
    pub fn new(config: MsrId) -> io::Result<Self> {
        match MSR_TYPE.load(Ordering::Relaxed) {
            0 => {
                let text = std::fs::read_to_string("/sys/bus/event_source/devices/msr/type")?;
                let ty = text
                    .trim_end()
                    .parse()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                MSR_TYPE.store(ty, Ordering::Relaxed);
                Ok(Self { ty, config })
            }
            ty => return Ok(Self { ty, config }),
        }
    }
}

impl Event for Msr {
    fn update_attrs(self, attr: &mut bindings::perf_event_attr) {
        attr.type_ = self.ty;
        attr.config = self.config.into();
    }
}
