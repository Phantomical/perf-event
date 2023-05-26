use std::io;

use c_enum::c_enum;
use perf_event_open_sys::bindings;

use crate::events::{CachedPmuType, Event};
use crate::Builder;

used_in_docs!(Builder);

static MSR_TYPE: CachedPmuType = CachedPmuType::new("msr");

c_enum! {
    /// Model-specific registers (MSRs) supported by the Linux msr PMU.
    ///
    /// Only some of these will be supported by any given system. To see which
    /// are supported on the current system you can look in the
    /// `/sys/bus/event_source/devices/msr/events` folder.
    ///
    /// The full list of MSR IDs supported by the msr PMU can be found within
    /// the [kernel source][src]. This enum aims to cover all entries supported
    /// by the kernel but may of course not be completedly up to date.
    ///
    /// Documentation on the underlying MSRs can be found in either the [intel
    /// software development manual][intel] or the [AMD architecture
    /// programmer's manual][amd].
    ///
    /// [src]: https://github.com/torvalds/linux/blob/master/arch/x86/events/msr.c
    /// [intel]: https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html
    /// [amd]: https://www.amd.com/system/files/TechDocs/40332.pdf
    #[derive(Clone, Copy, Eq, PartialEq, Hash)]
    pub enum MsrId: u64 {
        /// x86 Time Stamp Counter.
        ///
        /// This is the counter read by the `rdtsc` instruction. It is defined
        /// on all x86 CPUs.
        TSC = 0x0,

        /// x86 Actual Performance Frequency Clock (`MSR_IA32_APERF`).
        ///
        /// This counter increments in proportion to the actual CPU performance.
        /// Note that only the ratio between the `APERF` and `MPERF` counters
        /// has an architecturally defined meaning, not their absolute values.
        ///
        /// This counter is only available on Intel CPUs.
        APERF = 0x1,

        /// x86 Maximum Performance Frequency Clock Count (`MSR_IA32_MPERF`)
        ///
        /// This counter increments at a fixed frequency irrespective of CPU
        /// power state or frequency transitions. Note that only the ratio
        /// between the `APERF` and `MPERF` counters has an architecturally
        /// defined meaning, not their absolute values.
        ///
        /// You can use `APERF` and `MPERF` together to calculate the the
        /// average frequency of the CPU over the measurement period like so:
        /// ```text
        /// (APERF / MPERF) * CPU base frequency = running CPU frequency
        /// ```
        ///
        /// This counter is only available on Intel CPUs.
        MPERF = 0x2,

        /// Productive Performance Counter (`MSR_PPERF`).
        ///
        /// This counter is similar to `APERF` but only counts cycles perceived
        /// by the hardware as contributing to instruction execution (i.e. not
        /// halted and not stalled). This counter increments at teh same rate
        /// as `APERF` and the ratio `PPERF / APERF` can be used as an
        /// indicator of workload scalability.
        ///
        /// This counter is only available on Intel CPUs.
        PPERF = 0x3,

        /// System Management Interrupt Counter (`MSR_SMI_COUNT`).
        ///
        /// This counter counts the number of System Management Interrupts.
        ///
        /// Only available on Intel CPUs.
        SMI = 0x4,

        /// Performance Timestamp Counter (`MSR_F15H_PTSC`).
        ///
        /// This is a free-running counter that increments at a constant rate
        /// of 100MHz and is synchronized across all cores in a node to within
        /// +/-1.
        ///
        /// This counter is only available on AMD CPUs.
        PTSC = 0x5,

        /// Instructions Retired Performance Counter (`MSR_F17H_IRPERF`)
        ///
        /// This is a dedicated counter that always counts the number of
        /// instructions retired.
        ///
        /// This counter is only available on AMD CPUs.
        IRPERF = 0x6,

        /// Thermal Monitor Status (MSR_IA32_THERM_STATUS).
        ///
        /// This MSR provides status information about the thermal status of
        /// the current CPU core. It contains quite a bit of info so see the
        /// Intel SDM, Volume 3, section 15.8.2.5, for documentation on what
        /// it contains.
        ///
        /// This counter is only available on Intel CPUs.
        THERM = 0x7,
    }
}

/// A perf event providing access to a subset of the model-specific registers
/// (MSRs) on x86 CPUs.
///
/// The exact MSRs that are available on any given system depend on the system
/// CPU. The kernel exposes the available event types for the current
/// system under `/sys/bus/event_source/devices/msr/events`. You can use the
/// files in that folder to check if an MSR event is supported. Alternatively,
/// you can just try and build the counter, either will work.
///
/// The full list of MSR IDs that are supported can be seen [in the linux
/// source][0]. [`MsrId`] aims to be comprehensive but may not be up to date.
///
/// # Note
/// MSR events do not support filtering based on user-space vs kernel-space. You
/// will need to set [`exclude_kernel(false)`][exk] and
/// [`exclude_hv(false)`][exhv] or else you will get errors when calling
/// [`Builder::build`].
///
/// [0]: https://github.com/torvalds/linux/blob/master/arch/x86/events/msr.c
/// [exk]: crate::Builder::exclude_kernel
/// [exhv]: crate::Builder::exclude_hv
#[derive(Copy, Clone, Debug)]
pub struct Msr {
    ty: u32,
    id: MsrId,
}

impl Msr {
    /// Create a MSR event.
    ///
    /// # Errors
    /// This will attempt to read the PMU type from kernel device filesystem.
    /// Any errors due to missing files or folders will result in an error here.
    /// (e.g. should the kernel not have an MSR perf-event device).
    pub fn new(id: MsrId) -> io::Result<Self> {
        Ok(Self {
            ty: MSR_TYPE.get()?,
            id,
        })
    }
}

impl Event for Msr {
    fn update_attrs(self, attr: &mut bindings::perf_event_attr) {
        attr.type_ = self.ty;
        attr.config = self.id.into();
    }
}
