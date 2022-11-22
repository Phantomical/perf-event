//! Samples that the kernel can generate.
//!
//! This module contains bindings for samples emitted by the kernel into the
//! ringbuffer generated by `perf_event_open`. For authoritative documentation
//! on what each record means see the [`perf_event_open` manpage][man].
//!
//! The main type that you will need to use is the [`Record`] struct. It is
//! created by [`Sampler::next_record`] and represents a single event as
//! generated by the kernel.
//!
//! [`Sampler::next_record`]: crate::Sampler::next_record
//! [man]: https://man7.org/linux/man-pages/man2/perf_event_open.2.html

use bitflags::bitflags;
use bytes::Buf;
use perf_event_open_sys::bindings::{self, perf_event_attr, perf_event_header};
use std::fmt;

#[macro_use]
mod macros;

mod aux;
mod bpf_event;
mod cgroup;
mod comm;
mod exit;
mod fork;
mod itrace_start;
mod ksymbol;
mod lost;
mod lost_samples;
mod mmap;
mod mmap2;
mod namespaces;
mod read;
mod sample;
mod switch_cpu_wide;
mod text_poke;
mod throttle;

pub use self::aux::{Aux, AuxFlags};
pub use self::bitflags_defs::{ReadFormat, RecordMiscFlags, SampleType};
pub use self::bpf_event::{BpfEvent, BpfEventType};
pub use self::cgroup::Cgroup;
pub use self::comm::Comm;
pub use self::exit::Exit;
pub use self::fork::Fork;
pub use self::itrace_start::ITraceStart;
pub use self::ksymbol::{KSymbol, KSymbolFlags, KSymbolType};
pub use self::lost::Lost;
pub use self::lost_samples::LostSamples;
pub use self::mmap::Mmap;
pub use self::mmap2::Mmap2;
pub use self::namespaces::{NamespaceEntry, Namespaces};
pub use self::read::Read;
pub use self::sample::*;
pub use self::switch_cpu_wide::SwitchCpuWide;
pub use self::text_poke::TextPoke;
pub use self::throttle::Throttle;

// Need a module here to avoid the allow applying to everything.
#[allow(missing_docs)]
mod bitflags_defs {
    use super::*;

    bitflags! {
        /// Specifies which fields to include in the sample.
        ///
        /// These values correspond to `PERF_SAMPLE_x` values. See the
        /// [manpage] for documentation on what they mean.
        ///
        /// [`Sampler`]: crate::Sampler
        /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
        #[derive(Default)]
        pub struct SampleType : u64 {
            const IP = bindings::PERF_SAMPLE_IP;
            const TID = bindings::PERF_SAMPLE_TID;
            const TIME = bindings::PERF_SAMPLE_TIME;
            const ADDR = bindings::PERF_SAMPLE_ADDR;
            const READ = bindings::PERF_SAMPLE_READ;
            const CALLCHAIN = bindings::PERF_SAMPLE_CALLCHAIN;
            const ID = bindings::PERF_SAMPLE_ID;
            const CPU = bindings::PERF_SAMPLE_CPU;
            const PERIOD = bindings::PERF_SAMPLE_PERIOD;
            const STREAM_ID = bindings::PERF_SAMPLE_STREAM_ID;
            const RAW = bindings::PERF_SAMPLE_RAW;
            const BRANCH_STACK = bindings::PERF_SAMPLE_BRANCH_STACK;
            const REGS_USER = bindings::PERF_SAMPLE_REGS_USER;
            const STACK_USER = bindings::PERF_SAMPLE_STACK_USER;
            const WEIGHT = bindings::PERF_SAMPLE_WEIGHT;
            const DATA_SRC = bindings::PERF_SAMPLE_DATA_SRC;
            const IDENTIFIER = bindings::PERF_SAMPLE_IDENTIFIER;
            const TRANSACTION = bindings::PERF_SAMPLE_TRANSACTION;
            const REGS_INTR = bindings::PERF_SAMPLE_REGS_INTR;
            const PHYS_ADDR = bindings::PERF_SAMPLE_PHYS_ADDR;
            const AUX = bindings::PERF_SAMPLE_AUX;
            const CGROUP = bindings::PERF_SAMPLE_CGROUP;

            // The following are present in perf_event.h but not yet documented
            // in the manpage.
            const DATA_PAGE_SIZE = bindings::PERF_SAMPLE_DATA_PAGE_SIZE;
            const CODE_PAGE_SIZE = bindings::PERF_SAMPLE_CODE_PAGE_SIZE;
            const WEIGHT_STRUCT = bindings::PERF_SAMPLE_WEIGHT_STRUCT;

            // Don't clobber unknown flags when constructing the bitflag struct.
            #[doc(hidden)]
            const _ALLOW_ALL_FLAGS = !0;
        }
    }

    bitflags! {
        /// Bitfield specifying which fields are returned when reading the counter.
        ///
        /// See the [manpage] for documentation on what each flag means.
        ///
        /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
        #[derive(Default)]
        pub struct ReadFormat : u64 {
            const TOTAL_TIME_ENABLED = bindings::PERF_FORMAT_TOTAL_TIME_ENABLED as _;
            const TOTAL_TIME_RUNNING = bindings::PERF_FORMAT_TOTAL_TIME_RUNNING as _;
            const ID = bindings::PERF_FORMAT_ID as _;
            const GROUP = bindings::PERF_FORMAT_GROUP as _;
        }
    }

    bitflags! {
        /// Additional flags about the record event.
        ///
        /// Not all of these apply for every record type and in certain cases the
        /// same bit is reused to mean different things for different record types.
        ///
        /// See the [manpage] for documentation on what each flag means.
        ///
        /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
        #[derive(Default)]
        pub struct RecordMiscFlags : u16 {
            /// The first few bytes of these flags actually contain an enum value.
            ///
            /// Use [`cpumode`](Self::cpumode) to access them.
            const CPUMODE_MASK = bindings::PERF_RECORD_MISC_CPUMODE_MASK as _;

            /// Indicates that the associated [`Mmap`] or [`Mmap2`] record is
            /// for a non-executable memory mapping.
            const MMAP_DATA = bindings::PERF_RECORD_MISC_MMAP_DATA as _;

            /// Indicates that the [`Comm`] record is due to an `exec` syscall.
            const COMM_EXEC = bindings::PERF_RECORD_MISC_COMM_EXEC as _;

            /// Indicates that the context switch event was away from the
            /// process ID contained within the sample.
            const SWITCH_OUT = bindings::PERF_RECORD_MISC_SWITCH_OUT as _;

            /// Indicates that the contents of `Sample::ip` points to the
            /// exact instruction that generated the event.
            const EXACT_IP = bindings::PERF_RECORD_MISC_EXACT_IP as _;

            const EXT_RESERVED = bindings::PERF_RECORD_MISC_EXT_RESERVED as _;

            // New flags will likely be added to the perf_event_open interface in
            // the future. In that case we would like to avoid deleting those flags.
            // This field will ensure that the bitflags crate does not truncate any
            // flags when we construct a RecordMiscFlags instance.
            #[doc(hidden)]
            const _ALLOW_ALL_FLAGS = u16::MAX;
        }
    }

    impl SampleType {
        /// Create a sample from the underlying bits.
        pub const fn new(bits: u64) -> Self {
            Self { bits }
        }
    }

    /// Create a new read format from the underlying bits.
    impl ReadFormat {
        pub const fn new(bits: u64) -> Self {
            Self { bits }
        }
    }

    impl RecordMiscFlags {
        /// Create a set of flags from the underlying bits.
        pub const fn new(bits: u16) -> Self {
            Self { bits }
        }
    }
}

enum_binding! {
    /// The type of the record as communicated by the kernel.
    pub struct RecordType : u32 {
        const MMAP = bindings::PERF_RECORD_MMAP;
        const LOST = bindings::PERF_RECORD_LOST;
        const COMM = bindings::PERF_RECORD_COMM;
        const EXIT = bindings::PERF_RECORD_EXIT;
        const THROTTLE = bindings::PERF_RECORD_THROTTLE;
        const UNTHROTTLE = bindings::PERF_RECORD_UNTHROTTLE;
        const FORK = bindings::PERF_RECORD_FORK;
        const READ = bindings::PERF_RECORD_READ;
        const SAMPLE = bindings::PERF_RECORD_SAMPLE;
        const MMAP2 = bindings::PERF_RECORD_MMAP2;
        const AUX = bindings::PERF_RECORD_AUX;
        const ITRACE_START = bindings::PERF_RECORD_ITRACE_START;
        const LOST_SAMPLES = bindings::PERF_RECORD_LOST_SAMPLES;
        const SWITCH = bindings::PERF_RECORD_SWITCH;
        const SWITCH_CPU_WIDE = bindings::PERF_RECORD_SWITCH_CPU_WIDE;
        const NAMESPACES = bindings::PERF_RECORD_NAMESPACES;
        const KSYMBOL = bindings::PERF_RECORD_KSYMBOL;
        const BPF_EVENT = bindings::PERF_RECORD_BPF_EVENT;
        const CGROUP = bindings::PERF_RECORD_CGROUP;
        const TEXT_POKE = bindings::PERF_RECORD_TEXT_POKE;
    }
}

enum_binding! {
    /// ABI of the program when sampling registers.
    pub struct SampleRegsAbi : u64 {
        const NONE = bindings::PERF_SAMPLE_REGS_ABI_NONE as _;
        const ABI_32 = bindings::PERF_SAMPLE_REGS_ABI_32 as _;
        const ABI_64 = bindings::PERF_SAMPLE_REGS_ABI_64 as _;
    }
}

enum_binding! {
    /// Indicates the CPU mode in which the sample was collected.
    ///
    /// See the [manpage] for the documentation of what each value means.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub struct RecordCpuMode : u16 {
        const UNKNOWN = bindings::PERF_RECORD_MISC_CPUMODE_UNKNOWN as _;
        const KERNEL = bindings::PERF_RECORD_MISC_KERNEL as _;
        const USER = bindings::PERF_RECORD_MISC_USER as _;
        const HYPERVISOR = bindings::PERF_RECORD_MISC_HYPERVISOR as _;
        const GUEST_KERNEL = bindings::PERF_RECORD_MISC_GUEST_KERNEL as _;
        const GUEST_USER = bindings::PERF_RECORD_MISC_GUEST_USER as _;
    }
}

/// An event emitted by the kernel.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Record {
    /// Indicates which type of event was emitted by the kernel.
    ///
    /// Most of the time you will not need to use this. However, if you run
    /// into events which are not supported by perf-event then this should
    /// give you the ability to parse them from the [`RecordEvent::Unknown`]
    /// variant.
    pub ty: RecordType,

    /// Contains additional inforamtion about the sample.
    pub misc: RecordMiscFlags,

    /// The actual event as emitted by `perf_event_open`.
    pub event: RecordEvent,

    /// If `sample_id_all` is set when creating the sampler then this field
    /// will contain a subset of the selected sample fields.
    pub sample_id: SampleId,
}

/// A subset of the sample fields attached to every event.
///
/// If `sample_id_all` is set when creating the [`Sampler`][crate::Sampler]
/// instance then this struct will contain selected fields related to where
/// and when an event took place.
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct SampleId {
    /// The process ID of the process which generated the event.
    pub pid: Option<u32>,

    /// The thread ID of the thread which generated the event.
    pub tid: Option<u32>,

    /// The time at which the event was generated.
    pub time: Option<u64>,

    /// An ID which uniquely identifies the counter.
    ///
    /// If the counter that generated this event was a member of a group, then
    /// this will be the ID of the group leader instead.
    pub id: Option<u64>,

    /// An ID which uniquely identifies the counter.
    ///
    /// If the counter that generated this event is a member of a group, then
    /// this will still be the member of the counter and not the group leader.
    pub stream_id: Option<u64>,

    /// The CPU on which the event was generated.
    pub cpu: Option<u32>,
}

/// The data specific to the record event type.
///
/// If the event type is not supported by `perf-event` then it will return
/// [`RecordEvent::Unknown`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum RecordEvent {
    /// Record of a new memory map.
    Mmap(Mmap),

    /// Record indicating that the kernel dropped some events.
    Lost(Lost),

    /// Record indicating that a process name changed.
    Comm(Comm),

    /// Record indicating that a process exited.
    Exit(Exit),

    /// Record indicating that a process forked.
    Fork(Fork),

    /// Record indicating that the kernel disabled the generation of sample
    /// events due to too many being emitted in a single timer tick.
    Throttle(Throttle),

    /// Record indicating that the kernel has re-enabled the generation of
    /// sample events after the counter was throttled.
    Unthrottle(Throttle),

    /// Record indicating a read event.
    Read(Read),

    /// Record containing data about a sample taken by the kernel.
    Sample(Sample),

    /// Record a new memory map with extended info.
    Mmap2(Mmap2),

    /// Record indicating that there is new data in the aux buffer.
    Aux(Aux),

    /// Record indicating that a process has started an instruction trace.
    ITraceStart(ITraceStart),

    /// Record indicating that some samples were lost while using hardware
    /// sampling.
    LostSamples(LostSamples),

    /// Record that a context switch occurred.
    ///
    /// The [`RecordMiscFlags::SWITCH_OUT`] flag indicates whether this was due
    /// to a context into the current process or away from it.
    Switch,

    /// Record for a context switch in cpu-wide mode.
    ///
    /// This includes some additional information not contained in the regular
    /// [`Switch`](Self::Switch) record.
    SwitchCpuWide(SwitchCpuWide),

    /// Record containing namespace information about a process.
    Namespaces(Namespaces),

    /// Record for when a kernel symbol is being registered or unregistered.
    KSymbol(KSymbol),

    /// Record for when a BPF program is loaded or unloaded.
    BpfEvent(BpfEvent),

    /// Record for when a new cgroup is created and activated.
    Cgroup(Cgroup),

    /// Record for when kernel text is modified.
    TextPoke(TextPoke),

    /// An event was generated but `perf-event` was not able to parse it.
    ///
    /// Instead, the bytes making up the event are available here.
    Unknown(Vec<u8>),
}

/// Value of a counter along with some additional info.
#[derive(Copy, Clone, Debug)]
pub struct CounterValue {
    /// The value of the counter.
    pub value: u64,

    /// The number of nanoseconds for which this counter was enabled.
    pub time_enabled: Option<u64>,

    /// The number of nanoseconds for which this counter was both enabled and
    /// being updated.
    pub time_running: Option<u64>,

    /// Unique ID that identifies which stream this is from.
    pub id: Option<u64>,
}

/// The values of all counters in a group.
#[derive(Clone, Debug)]
pub struct GroupValue {
    /// The number of nanoseconds for which this counter group was enabled.
    pub time_enabled: Option<u64>,

    /// The number of nanoseconds that this counter group was both enabled and
    /// being updated.
    pub time_running: Option<u64>,

    /// The values of all counters within the group.
    pub values: Vec<GroupValueEntry>,
}

/// The value of a single counter within a counter group.
#[derive(Copy, Clone, Debug)]
pub struct GroupValueEntry {
    /// The count of events as recorded by the counter.
    pub value: u64,

    /// A unique ID for this counter.
    pub id: Option<u64>,
}

/// A value as part of a `RecordEvent::Sample` or `RecordEvent::Read`.
///
/// This corresponds to the `read_format` struct in the [manpage].
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub enum ReadValue {
    Counter(CounterValue),
    Group(GroupValue),
}

/// All the config info needed to parse a record from the perf ring buffer.
///
/// If you need something new, add it here!
#[derive(Default)]
pub(crate) struct ParseConfig {
    sample_type: SampleType,
    read_format: ReadFormat,
    sample_id_all: bool,
    regs_user: u64,
    regs_intr: u64,
}

impl Record {
    pub(crate) fn parse<B>(config: &ParseConfig, header: &perf_event_header, buf: &mut B) -> Self
    where
        B: Buf,
    {
        let ty = header.type_.into();
        let sample_id_len = match ty {
            // MMAP and SAMPLE do not include the sample_id trailer
            RecordType::MMAP | RecordType::SAMPLE => None,
            _ => Some(SampleId::expected_size(config)),
        };

        let mut limited = buf.take(buf.remaining() - sample_id_len.unwrap_or(0));
        let event = match ty {
            RecordType::MMAP => Mmap::parse(config, &mut limited).into(),
            RecordType::LOST => Lost::parse(config, &mut limited).into(),
            RecordType::COMM => Comm::parse(config, &mut limited).into(),
            RecordType::EXIT => Exit::parse(config, &mut limited).into(),
            RecordType::THROTTLE => RecordEvent::Throttle(Throttle::parse(config, &mut limited)),
            RecordType::UNTHROTTLE => {
                RecordEvent::Unthrottle(Throttle::parse(config, &mut limited))
            }
            RecordType::FORK => Fork::parse(config, &mut limited).into(),
            RecordType::READ => Read::parse(config, &mut limited).into(),
            RecordType::SAMPLE => Sample::parse(config, &mut limited).into(),
            RecordType::MMAP2 => Mmap2::parse(config, &mut limited).into(),
            RecordType::AUX => Aux::parse(config, &mut limited).into(),
            RecordType::ITRACE_START => ITraceStart::parse(config, &mut limited).into(),
            RecordType::LOST_SAMPLES => LostSamples::parse(config, &mut limited).into(),
            RecordType::SWITCH => RecordEvent::Switch,
            RecordType::SWITCH_CPU_WIDE => {
                SwitchCpuWide::parse(RecordMiscFlags::new(header.misc), &mut limited).into()
            }
            RecordType::NAMESPACES => Namespaces::parse(config, &mut limited).into(),
            RecordType::KSYMBOL => KSymbol::parse(config, &mut limited).into(),
            RecordType::BPF_EVENT => BpfEvent::parse(config, &mut limited).into(),
            RecordType::CGROUP => Cgroup::parse(config, &mut limited).into(),
            RecordType::TEXT_POKE => TextPoke::parse(config, &mut limited).into(),
            _ => RecordEvent::Unknown(limited.parse_remainder()),
        };

        limited.advance(limited.remaining());

        let sample_id = match sample_id_len {
            Some(_) => SampleId::parse(config, buf),
            // Fill in some fields from the record in cases where there is no
            // sample_id encoded with the record.
            None => match &event {
                RecordEvent::Mmap(mmap) => SampleId {
                    pid: Some(mmap.pid),
                    tid: Some(mmap.tid),
                    ..Default::default()
                },
                RecordEvent::Sample(sample) => SampleId {
                    pid: sample.pid,
                    tid: sample.tid,
                    time: sample.time,
                    id: sample.id,
                    stream_id: sample.stream_id,
                    cpu: sample.cpu,
                },
                _ => SampleId::default(),
            },
        };

        Self {
            ty,
            misc: RecordMiscFlags::new(header.misc),
            event,
            sample_id,
        }
    }
}

impl SampleId {
    fn expected_size(config: &ParseConfig) -> usize {
        if !config.sample_id_all {
            return 0;
        }

        let configs = [
            config.sample_type.contains(SampleType::TID),
            config.sample_type.contains(SampleType::TIME),
            config.sample_type.contains(SampleType::ID),
            config.sample_type.contains(SampleType::STREAM_ID),
            config.sample_type.contains(SampleType::CPU),
            config.sample_type.contains(SampleType::IDENTIFIER),
        ];

        configs.iter().copied().filter(|&x| x).count() * std::mem::size_of::<u64>()
    }
}

impl fmt::Debug for SampleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("SampleId");
        let mut fieldcnt = 0;

        if let Some(pid) = &self.pid {
            dbg.field("pid", pid);
            fieldcnt += 1;
        }

        if let Some(tid) = &self.tid {
            dbg.field("tid", tid);
            fieldcnt += 1;
        }

        if let Some(time) = &self.time {
            dbg.field("time", time);
            fieldcnt += 1;
        }

        if let Some(id) = &self.id {
            dbg.field("id", id);
            fieldcnt += 1;
        }

        if let Some(stream_id) = &self.stream_id {
            dbg.field("stream_id", stream_id);
            fieldcnt += 1;
        }

        if let Some(cpu) = &self.cpu {
            dbg.field("cpu", cpu);
            fieldcnt += 1;
        }

        if fieldcnt == 6 {
            dbg.finish()
        } else {
            dbg.finish_non_exhaustive()
        }
    }
}

impl From<&'_ perf_event_attr> for ParseConfig {
    fn from(attr: &perf_event_attr) -> Self {
        Self {
            sample_type: SampleType::new(attr.sample_type),
            read_format: ReadFormat::new(attr.read_format),
            sample_id_all: attr.sample_id_all() != 0,
            regs_user: attr.sample_regs_user,
            regs_intr: attr.sample_regs_intr,
        }
    }
}

impl From<perf_event_attr> for ParseConfig {
    fn from(attr: perf_event_attr) -> Self {
        Self::from(&attr)
    }
}

impl RecordMiscFlags {
    /// Returns the CPU mode bits.
    pub fn cpumode(&self) -> RecordCpuMode {
        (*self & Self::CPUMODE_MASK).bits().into()
    }
}

/// Trait for types which are parseable given the necessary configuration
/// context.
pub(crate) trait Parse {
    fn parse<B: Buf>(config: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized;
}

impl Parse for SampleId {
    fn parse<B: Buf>(config: &ParseConfig, buf: &mut B) -> Self {
        if !config.sample_id_all {
            return Self::default();
        }

        let mut sample = Self::default();
        if config.sample_type.contains(SampleType::TID) {
            sample.pid = Some(buf.get_u32_ne());
            sample.tid = Some(buf.get_u32_ne());
        }

        if config.sample_type.contains(SampleType::TIME) {
            sample.time = Some(buf.get_u64_ne());
        }

        if config.sample_type.contains(SampleType::ID) {
            sample.id = Some(buf.get_u64_ne());
        }

        if config.sample_type.contains(SampleType::STREAM_ID) {
            sample.stream_id = Some(buf.get_u64_ne());
        }

        if config.sample_type.contains(SampleType::CPU) {
            sample.cpu = Some(buf.get_u32_ne());
            let _ = buf.get_u32_ne(); // res
        }

        if config.sample_type.contains(SampleType::IDENTIFIER) {
            sample.id = Some(buf.get_u64_ne());
        }

        sample
    }
}

impl Parse for CounterValue {
    fn parse<B: Buf>(config: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            value: buf.get_u64_ne(),
            time_enabled: config
                .read_format
                .contains(ReadFormat::TOTAL_TIME_ENABLED)
                .then(|| buf.get_u64_ne()),
            time_running: config
                .read_format
                .contains(ReadFormat::TOTAL_TIME_RUNNING)
                .then(|| buf.get_u64_ne()),
            id: config
                .read_format
                .contains(ReadFormat::ID)
                .then(|| buf.get_u64_ne()),
        }
    }
}

impl Parse for GroupValue {
    fn parse<B: Buf>(config: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        let len = buf.get_u64_ne() as usize;

        Self {
            time_enabled: config
                .read_format
                .contains(ReadFormat::TOTAL_TIME_ENABLED)
                .then(|| buf.get_u64_ne()),
            time_running: config
                .read_format
                .contains(ReadFormat::TOTAL_TIME_RUNNING)
                .then(|| buf.get_u64_ne()),
            values: std::iter::repeat_with(|| GroupValueEntry::parse(config, buf))
                .take(len)
                .collect(),
        }
    }
}

impl Parse for GroupValueEntry {
    fn parse<B: Buf>(config: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self {
            value: buf.get_u64_ne(),
            id: config
                .read_format
                .contains(ReadFormat::ID)
                .then(|| buf.get_u64_ne()),
        }
    }
}

impl Parse for ReadValue {
    fn parse<B: Buf>(config: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        if config.read_format.contains(ReadFormat::GROUP) {
            Self::Group(GroupValue::parse(config, buf))
        } else {
            Self::Counter(CounterValue::parse(config, buf))
        }
    }
}

/// Utility trait for parsing data out of a [`Buf`] without panicking.
pub(crate) trait ParseBuf: Buf {
    fn parse_vec(&mut self, mut len: usize) -> Vec<u8> {
        assert!(len <= self.remaining());

        let mut vec = Vec::with_capacity(len);

        while len > 0 {
            let chunk = self.chunk();
            let chunk = &chunk[..len.min(chunk.len())];
            vec.extend_from_slice(chunk);
            len -= chunk.len();
            self.advance(chunk.len());
        }

        vec
    }

    /// Parse the remaining bytes within the buffer to a Vec.
    fn parse_remainder(&mut self) -> Vec<u8> {
        self.parse_vec(self.remaining())
    }

    /// Parse a constant number of bytes to an array.
    fn parse_bytes<const N: usize>(&mut self) -> [u8; N] {
        assert!(N <= self.remaining());

        let mut bytes = [0; N];
        self.copy_to_slice(&mut bytes);
        bytes
    }

    /// Parse a type by copying its bytes out of the buffer and transmuting
    /// to the desired type.
    ///
    /// # Safety
    /// It must be valid to transmute T from any arbitrary set of initialized
    /// bytes.
    unsafe fn parse_transmute<T: Copy>(&mut self) -> T {
        use std::mem::MaybeUninit;
        use std::slice::from_raw_parts_mut;

        let mut value = MaybeUninit::<T>::zeroed();
        let slice = from_raw_parts_mut(value.as_mut_ptr() as *mut u8, std::mem::size_of::<T>());
        self.copy_to_slice(slice);

        value.assume_init()
    }

    fn parse_header(&mut self) -> bindings::perf_event_header {
        unsafe { self.parse_transmute() }
    }
}

impl<B: Buf> ParseBuf for B {}

#[cfg(test)]
mod tests {
    use crate::Builder;

    use super::*;

    #[test]
    fn sample_id_expected_size_empty_with_flags() {
        let builder = Builder::new()
            .sample(SampleType::CPU)
            .sample(SampleType::TID);
        let config = ParseConfig::from(builder.attrs);

        // sample_id_all not specified so size should be 0
        assert_eq!(SampleId::expected_size(&config), 0);
    }

    #[test]
    fn sample_id_parse_empty_with_flags() {
        let builder = Builder::new()
            .sample(SampleType::CPU)
            .sample(SampleType::TID);
        let config = ParseConfig::from(builder.attrs);
        let mut buf: &[u8] = &[];

        // sample_id_all not specified so nothing should be parsed
        let sample_id = SampleId::parse(&config, &mut buf);

        assert_eq!(sample_id.cpu, None);
        assert_eq!(sample_id.tid, None);
        assert_eq!(sample_id.pid, None);
    }

    #[test]
    #[cfg_attr(target_endian = "big", ignore = "requires a little-endian target")]
    fn sample_id_all_parse_full_with_flags() {
        let builder = Builder::new()
            .sample(SampleType::CPU)
            .sample(SampleType::TIME);
        let mut config = ParseConfig::from(builder.attrs);
        // Don't have a method for setting this on the builder yet so do it
        // directly here.
        config.sample_id_all = true;

        #[rustfmt::skip]
        let mut buf: &[u8] = &[
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];

        let sample_id = SampleId::parse(&config, &mut buf);

        assert_eq!(buf.len(), 0);

        assert_eq!(sample_id.time, Some(1));
        assert_eq!(sample_id.cpu, Some(2));
        assert_eq!(sample_id.pid, None);
        assert_eq!(sample_id.tid, None);
        assert_eq!(sample_id.id, None);
    }
}
