use std::borrow::Cow;
use std::convert::{AsMut, AsRef};
use std::ops::{Deref, DerefMut};
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::data::parse::{ParseBuf, ParseBufChunk, ParseError, ParseResult, Parser};
use crate::events::Hardware;
use crate::sys::bindings::{
    __BindgenBitfieldUnit, perf_event_header, perf_event_mmap_page,
    perf_event_mmap_page__bindgen_ty_1__bindgen_ty_1 as MmapPageFlags,
};
use crate::{check_errno_syscall, data, Counter};

used_in_docs!(Hardware);

/// A sampled perf event.
///
/// A sampler for a sampler perf event consists of two things: a [`Counter`],
/// and a memory-mapped ring buffer into which the kernel periodically writes
/// events. The specific event is configured on construction and can vary from
/// changes to the memory mapping associated with a process, to sampling call
/// stacks, to getting the output from a bpf program running in the kernel, and
/// more.
///
/// This sampler type provides direct access to the bytes written by the kernel
/// without doing any parsing of the emitted records. To actually read the
/// involved fields you will need to parse them yourself. See the
/// [`perf_event_open` man page][0] for documentation on how the sample records
/// are represented in memory.
///
/// [0]: https://www.mankier.com/2/perf_event_open
pub struct Sampler {
    counter: Counter,
    mmap: memmap2::MmapRaw,
}

/// A view into a [`Sampler`]'s ring buffer for a single kernel event record.
///
/// When dropped, this type will advance the tail pointer in the ringbuffer of
/// the [`Sampler`] that it references. To avoid this, you can use
/// [`std::mem::forget`] so the next call to [`Sampler::next_record`] will
/// return the same record again.
pub struct Record<'a> {
    sampler: &'a Sampler,
    header: perf_event_header,
    data: ByteBuffer<'a>,
}

/// A `Buf` that can be either a single byte slice or two disjoint byte
/// slices.
#[derive(Copy, Clone)]
enum ByteBuffer<'a> {
    Single(&'a [u8]),
    Split([&'a [u8]; 2]),
}

impl Sampler {
    pub(crate) fn new(counter: Counter, mmap: memmap2::MmapRaw) -> Self {
        assert!(!mmap.as_ptr().is_null());

        Self { counter, mmap }
    }

    /// Convert this sampler back into a counter.
    ///
    /// This will close the ringbuffer associated with the sampler.
    pub fn into_counter(self) -> Counter {
        self.counter
    }

    /// Access the underlying counter for this sampler.
    pub fn as_counter(&self) -> &Counter {
        &self.counter
    }

    /// Mutably access the underlying counter for this sampler.
    pub fn as_counter_mut(&mut self) -> &mut Counter {
        &mut self.counter
    }

    /// Read the next record from the ring buffer.
    ///
    /// This method does not block. If you want blocking behaviour, use
    /// [`next_blocking`] instead.
    ///
    /// It is possible to get readiness notifications for when events are
    /// present in the ring buffer (e.g. for async code). See the documentation
    /// on the [`perf_event_open`][man] manpage for details on how to do this.
    ///
    /// [`next_blocking`]: Self::next_blocking
    /// [man]: https://www.mankier.com/2/perf_event_open
    pub fn next_record(&mut self) -> Option<Record> {
        use std::{mem, ptr, slice};

        let page = self.page();

        // SAFETY:
        // - page points to a valid instance of perf_event_mmap_page.
        // - data_tail is only written by the user side so it is safe to do a non-atomic
        //   read here.
        let tail = unsafe { ptr::read(ptr::addr_of!((*page).data_tail)) };
        // ATOMICS:
        // - The acquire load here syncronizes with the release store in the kernel and
        //   ensures that all the data written to the ring buffer before data_head is
        //   visible to this thread.
        // SAFETY:
        // - page points to a valid instance of perf_event_mmap_page.
        let head = unsafe { atomic_load(ptr::addr_of!((*page).data_head), Ordering::Acquire) };

        if tail == head {
            return None;
        }

        // SAFETY: (for both statements)
        // - page points to a valid instance of perf_event_mmap_page.
        // - neither of these fields are written to except before the map is created so
        //   reading from them non-atomically is safe.
        let data_size = unsafe { ptr::read(ptr::addr_of!((*page).data_size)) };
        let data_offset = unsafe { ptr::read(ptr::addr_of!((*page).data_offset)) };

        let mod_tail = (tail % data_size) as usize;
        let mod_head = (head % data_size) as usize;

        // SAFETY:
        // - perf_event_open guarantees that page.data_offset is within the memory
        //   mapping.
        let data_start = unsafe { self.mmap.as_ptr().add(data_offset as usize) };
        // SAFETY:
        // - data_start is guaranteed to be valid for at least data_size bytes.
        let tail_start = unsafe { data_start.add(mod_tail) };

        let mut buffer = if mod_head > mod_tail {
            ByteBuffer::Single(unsafe { slice::from_raw_parts(tail_start, mod_head - mod_tail) })
        } else {
            ByteBuffer::Split([
                unsafe { slice::from_raw_parts(tail_start, data_size as usize - mod_tail) },
                unsafe { slice::from_raw_parts(data_start, mod_head) },
            ])
        };

        let header = buffer.parse_header();
        assert!(header.size as usize >= mem::size_of::<perf_event_header>());
        buffer.truncate(header.size as usize - mem::size_of::<perf_event_header>());

        Some(Record {
            sampler: self,
            header,
            data: buffer,
        })
    }

    /// Read the next record from the ring buffer. This method will block (with
    /// an optional timeout) until a new record is available.
    ///
    /// If this sampler is only enabled for a single process and that process
    /// exits, this method will return `None` even if no timeout is passed.
    /// Note that this only works on Linux 3.18 and above.
    ///
    /// # Panics
    /// This method will panic if an unexpected error is returned from
    /// `libc::poll`. There are only two cases where this can happen:
    /// - the current process has run out of file descriptors, or,
    /// - the kernel couldn't allocate memory for internal poll datastructures.
    pub fn next_blocking(&mut self, timeout: Option<Duration>) -> Option<Record> {
        let deadline = timeout.map(|timeout| Instant::now() + timeout);

        loop {
            if let Some(record) = self.next_record() {
                // This is a workaround for a known limitation of NLL in rustc.
                // If it worked, we could do
                //    return Some(record);
                // but currently that extends the lifetime for the &mut self
                // borrow to cover the whole function and that causes conflicts
                // with other borrows further down.
                //
                // Fixing this is tracked in the following rustc issue
                // https://github.com/rust-lang/rust/issues/51132
                //
                // You can verify that the code above should, in fact, pass the
                // borrow checker by removing the line below, uncommenting the
                // line above, and checking it via
                //     cargo +nightly rustc -- -Zpolonius
                return Some(unsafe { std::mem::transmute::<Record, Record>(record) });
            }

            let timeout = match deadline {
                Some(deadline) => deadline
                    .checked_duration_since(Instant::now())?
                    .as_millis()
                    .min(libc::c_int::MAX as u128) as libc::c_int,
                None => -1,
            };

            let mut pollfd = libc::pollfd {
                fd: self.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };

            match check_errno_syscall(|| unsafe { libc::poll(&mut pollfd, 1, timeout) }) {
                // poll timed out.
                Ok(0) => return None,
                // The sampler was tracking a single other process and that
                // process has exited.
                //
                // However, there may still be events in the ring buffer in this case so
                // we still need to check.
                Ok(_) if pollfd.revents & libc::POLLHUP != 0 => return self.next_record(),
                // Must be POLLIN, there should be an event ready.
                Ok(_) => continue,
                Err(e) => match e.raw_os_error() {
                    Some(libc::EINTR) => continue,
                    // The only other possible kernel errors here are so rare
                    // that it doesn't make sense to make this API have a
                    // result because of them. To whit, they are:
                    // - EINVAL - the process ran out of file descriptors
                    // - ENOMEM - the kernel couldn't allocate memory for the poll datastructures.
                    // In this case, we panic.
                    _ => panic!(
                        "polling a perf-event fd returned an unexpected error: {}",
                        e
                    ),
                },
            }
        }
    }

    /// Read the value of this counter directly from userspace.
    ///
    /// Some CPU architectures allow performance counters to be read directly
    /// from userspace without having to go through the kernel. This can be much
    /// faster than a normal counter read but the tradeoff is that can only be
    /// done under certain conditions.
    ///
    /// This method allows you to read the counter value, `time_enabled`, and
    /// `time_running` without going through the kernel, if allowed by the
    /// combination of architecture, kernel, and counter. `time_enabled` and
    /// `time_running` are always read but will be less accurate on
    /// architectures that do not provide a timestamp counter readable from
    /// userspace.
    ///
    /// # Restrictions
    /// In order for counter values to be read using this method the following
    /// must be true:
    /// - the CPU architecture must support reading counters from userspace,
    /// - the counter must be recording for the current process,
    /// - perf-event2 must have support for the relevant CPU architecture, and,
    /// - the counter must correspond to a hardware counter.
    ///
    /// Note that, despite the above being true, the kernel may still not
    /// support userspace reads for other reasons. [`Hardware`] events should
    /// usually be supported but anything beyond that is unlikely. See the
    /// supported architectures table below to see which are supported by
    /// perf-event2.
    ///
    /// Accurate timestamps also require that the kernel, CPU, and perf-event2
    /// support them. They have similar restrictions to counter reads and will
    /// just return the base values set by the kernel otherwise. These will may
    /// be somewhat accurate but are likely to be out-of-date.
    ///
    /// # Supported Architectures
    /// | Architecture | Counter Read | Timestamp Read |
    /// |--------------|--------------|----------------|
    /// |  x86/x86_64  | yes          | yes            |
    ///
    /// If you would like to add support for a new architecture here please
    /// submit a PR!
    pub fn read_user(&self) -> UserReadData {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::_rdtsc;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::_rdtsc;

        loop {
            let mut data = unsafe { PmcReadData::new(self.page()) };

            if let Some(index) = data.index() {
                // SAFETY:
                // - index was handed to us by the kernel so it is safe to use.
                // - cap_user_rdpmc will only be set if it is valid to call rdpmc from
                //   userspace.
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                data.with_pmc(unsafe { rdpmc(index) });
            }

            if data.cap_user_time() {
                // SAFETY: it is always safe to run rdtsc on x86
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                data.with_tsc(unsafe { _rdtsc() });
            }

            if let Some(data) = data.finish() {
                return data;
            }
        }
    }

    fn page(&self) -> *const perf_event_mmap_page {
        self.mmap.as_ptr() as *const _
    }
}

impl Deref for Sampler {
    type Target = Counter;

    fn deref(&self) -> &Self::Target {
        &self.counter
    }
}

impl DerefMut for Sampler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.counter
    }
}

impl AsRef<Counter> for Sampler {
    fn as_ref(&self) -> &Counter {
        &self.counter
    }
}

impl AsMut<Counter> for Sampler {
    fn as_mut(&mut self) -> &mut Counter {
        &mut self.counter
    }
}

impl AsRawFd for Sampler {
    fn as_raw_fd(&self) -> RawFd {
        self.counter.as_raw_fd()
    }
}

impl IntoRawFd for Sampler {
    fn into_raw_fd(self) -> RawFd {
        self.counter.into_raw_fd()
    }
}

// This is meant to roughly be the equivalent of the kernel READ_ONCE
// macro. The closest equivalent in Rust (and, I think, the only one
// that avoids UB) is to do a relaxed atomic load.
//
// On x86 this translates to just a load from memory and it is still
// marked as an atomic for the compiler.
macro_rules! read_once {
    ($place:expr) => {
        atomic_load(::std::ptr::addr_of!($place), Ordering::Relaxed)
    };
}

// This is meant to be the equivalent of the kernel barrier macro. It prevents
// the compiler from reordering any memory accesses accross the barrier.
macro_rules! barrier {
    () => {
        std::sync::atomic::compiler_fence(Ordering::SeqCst)
    };
}

/// Helper for writing a `read_user` variant.
struct PmcReadData {
    page: *const perf_event_mmap_page,
    seq: u32,
    flags: MmapPageFlags,
    enabled: u64,
    running: u64,
    index: u32,
    count: i64,

    /// There are architectures that perf supports that we don't. On those
    /// architectures we don't want to just return the offset value from the
    /// perf mmap page.
    has_pmc_value: bool,
}

#[allow(dead_code)]
impl PmcReadData {
    /// Read the initial sequence number and other values out of the page.
    ///
    /// # Safety
    /// - `page` must point to a valid instance of [`perf_event_mmap_page`]
    /// - `page` must remain valid for the lifetime of this struct.
    pub unsafe fn new(page: *const perf_event_mmap_page) -> Self {
        let seq = atomic_load(std::ptr::addr_of!((*page).lock), Ordering::Acquire);
        barrier!();

        let capabilities = read_once!((*page).__bindgen_anon_1.capabilities);

        Self {
            page,
            seq,
            flags: {
                let mut flags = MmapPageFlags::default();
                flags._bitfield_1 = __BindgenBitfieldUnit::new(capabilities.to_ne_bytes());
                flags
            },
            enabled: read_once!((*page).time_enabled),
            running: read_once!((*page).time_running),

            index: read_once!((*page).index),
            count: read_once!((*page).offset),
            has_pmc_value: false,
        }
    }

    pub fn cap_user_rdpmc(&self) -> bool {
        self.flags.cap_user_rdpmc() != 0
    }

    pub fn cap_user_time(&self) -> bool {
        self.flags.cap_user_time() != 0
    }

    pub fn cap_user_time_short(&self) -> bool {
        self.flags.cap_user_time_short() != 0
    }

    /// Get the index of the PMC counter, should there be one to read.
    pub fn index(&self) -> Option<u32> {
        if self.cap_user_rdpmc() && self.index != 0 {
            Some(self.index - 1)
        } else {
            None
        }
    }

    /// Update the `enabled` and `running` counts using the tsc counter.
    ///
    /// # Panics
    /// Panics if `cap_user_time` is not true.
    pub fn with_tsc(&mut self, mut cyc: u64) {
        assert!(self.cap_user_time());

        // counter is not active so enabled and running should be accurate.
        if !self.cap_user_rdpmc() || self.index == 0 {
            return;
        }

        let page = self.page;

        let time_offset = unsafe { read_once!((*page).time_offset) };
        let time_mult = unsafe { read_once!((*page).time_mult) };
        let time_shift = unsafe { read_once!((*page).time_shift) };

        if self.cap_user_time_short() {
            let time_cycles = unsafe { read_once!((*page).time_cycles) };
            let time_mask = unsafe { read_once!((*page).time_mask) };

            cyc = time_cycles + ((cyc - time_cycles) & time_mask);
        }

        let time_mult = time_mult as u64;
        let quot = cyc >> time_shift;
        let rem = cyc & ((1u64 << time_shift) - 1);

        let delta = quot * time_mult + ((rem * time_mult) >> time_shift);
        let delta = time_offset.wrapping_add(delta);

        self.enabled += delta;
        if self.index != 0 {
            self.running += delta;
        }
    }

    /// Update the value of `count` using a value read from the architecture
    /// PMC.
    ///
    /// # Panics
    /// Panics if `index` return `None`.
    pub fn with_pmc(&mut self, pmc: u64) {
        assert!(self.index().is_some());

        let Self { page, .. } = *self;
        let width = unsafe { read_once!((*page).pmc_width) };

        let mut pmc = pmc as i64;
        pmc <<= 64 - width;
        pmc >>= 64 - width;

        self.count = self.count.wrapping_add(pmc);
        self.has_pmc_value = true;
    }

    pub fn finish(self) -> Option<UserReadData> {
        let page = self.page;
        let seq = self.seq;

        barrier!();
        let nseq = unsafe { atomic_load(std::ptr::addr_of!((*page).lock), Ordering::Acquire) };
        if nseq != seq {
            return None;
        }

        Some(UserReadData {
            time_enabled: self.enabled,
            time_running: self.running,
            value: if self.has_pmc_value {
                Some(self.count as u64)
            } else {
                None
            },
        })
    }
}

/// Data read from a call to [`Sampler::read_user`].
#[derive(Copy, Clone, Debug)]
pub struct UserReadData {
    time_enabled: u64,
    time_running: u64,
    value: Option<u64>,
}

impl UserReadData {
    /// The total time for which the counter was enabled at the time of reading.
    ///
    /// If the architecture and counter support it this will be cycle-accurate
    pub fn time_enabled(&self) -> Duration {
        Duration::from_nanos(self.time_enabled)
    }

    /// The total time for which the counter was running at the time of reading.
    pub fn time_running(&self) -> Duration {
        Duration::from_nanos(self.time_running)
    }

    /// The value of the counter, if it was enabled at the time.
    pub fn count(&self) -> Option<u64> {
        self.value
    }

    /// The value of the counter, scaled to reflect `time_enabled`.
    pub fn scaled_count(&self) -> Option<u64> {
        self.count().map(|count| {
            let quot = count / self.time_running;
            let rem = count % self.time_running;
            quot * self.time_enabled + (rem * self.time_enabled) / self.time_running
        })
    }
}

impl<'s> Record<'s> {
    /// Access the `type` field of the kernel record header.
    ///
    /// This indicates the type of the record emitted by the kernel.
    pub fn ty(&self) -> u32 {
        self.header.type_
    }

    /// Access the `misc` field of the kernel record header.
    ///
    /// This contains a set of flags that carry some additional metadata on the
    /// record being emitted by the kernel.
    pub fn misc(&self) -> u16 {
        self.header.misc
    }

    /// Get the total length, in bytes, of this record.
    #[allow(clippy::len_without_is_empty)] // Records are never empty
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Access the bytes of this record.
    ///
    /// Since the underlying buffer is a ring buffer the bytes of the record
    /// may end up wrapping around the end of the buffer. That gets exposed
    /// here as data returning either one or two byte slices. If there is no
    /// wrap-around then one slice will be returned here, otherwise, two will
    /// be returned.
    pub fn data(&self) -> &[&[u8]] {
        match &self.data {
            ByteBuffer::Single(buf) => std::slice::from_ref(buf),
            ByteBuffer::Split(bufs) => &bufs[..],
        }
    }

    /// Copy the bytes of this record to an owned [`Vec`].
    pub fn to_vec(&self) -> Vec<u8> {
        self.to_contiguous().into_owned()
    }

    /// Get the bytes of this record as a single contiguous slice.
    ///
    /// For most records this is effectively free but if the record wraps
    /// around the end of the ringbuffer then it will be copied to a vector.
    pub fn to_contiguous(&self) -> Cow<[u8]> {
        match self.data {
            ByteBuffer::Single(data) => Cow::Borrowed(data),
            ByteBuffer::Split([a, b]) => {
                let mut vec = Vec::with_capacity(a.len() + b.len());
                vec.extend_from_slice(a);
                vec.extend_from_slice(b);
                Cow::Owned(vec)
            }
        }
    }

    /// Parse the data in this record to a [`data::Record`] enum.
    pub fn parse_record(&self) -> ParseResult<data::Record> {
        let mut parser = Parser::new(self.data, self.sampler.config().clone());
        data::Record::parse_with_header(&mut parser, self.header)
    }

    /// Parse the sample id for the record.
    ///
    /// This will only be non-empty if the [`sample_id_all`] was set when
    /// building the counter. In addition, `MMAP` records never have a sample id
    /// set. If you want sample ids and `MMAP` records you will need to request
    /// `MMAP2` records instead.
    ///
    /// [`sample_id_all`]: crate::Builder::sample_id_all
    pub fn parse_sample_id(&self) -> ParseResult<data::SampleId> {
        use perf_event_open_sys::bindings;

        let config = self.sampler.config();
        let mut parser = Parser::new(self.data, config.clone());

        let (mut parser, metadata) = parser.parse_metadata_with_header(self.header)?;

        // All other records either already parsed the sample id or don't have it.
        // With SAMPLE records, we can construct the sample id struct directly.
        if self.ty() != bindings::PERF_RECORD_SAMPLE {
            return Ok(metadata.sample_id().clone());
        }

        let record = parser.parse::<data::Sample>()?;
        Ok(data::SampleId::from_sample(&record))
    }
}

impl<'s> Drop for Record<'s> {
    fn drop(&mut self) {
        use std::ptr;

        let page = self.sampler.page();

        unsafe {
            // SAFETY:
            // - page points to a valid instance of perf_event_mmap_page
            // - data_tail is only written on our side so it is safe to do a non-atomic read
            //   here.
            let tail = ptr::read(ptr::addr_of!((*page).data_tail));

            // ATOMICS:
            // - The release store here prevents the compiler from re-ordering any reads
            //   past the store to data_tail.
            // SAFETY:
            // - page points to a valid instance of perf_event_mmap_page
            atomic_store(
                ptr::addr_of!((*page).data_tail),
                tail + (self.header.size as u64),
                Ordering::Release,
            );
        }
    }
}

// Record contains a pointer which prevents it from implementing Send or Sync
// by default. It is, however, valid to send it across threads and it has no
// interior mutability so we implement Send and Sync here manually.
unsafe impl<'s> Sync for Record<'s> {}
unsafe impl<'s> Send for Record<'s> {}

impl<'a> ByteBuffer<'a> {
    /// Parse an instance of `perf_event_header` out of the start of this
    /// byte buffer.
    fn parse_header(&mut self) -> perf_event_header {
        let mut bytes = [0; std::mem::size_of::<perf_event_header>()];
        self.copy_to_slice(&mut bytes);
        // SAFETY: perf_event_header is a packed C struct so it is valid to
        //         copy arbitrary initialized memory into it.
        unsafe { std::mem::transmute(bytes) }
    }

    fn len(&self) -> usize {
        match self {
            Self::Single(buf) => buf.len(),
            Self::Split([a, b]) => a.len() + b.len(),
        }
    }

    /// Shorten this byte buffer to only include the first `new_len` bytes.
    ///
    /// # Panics
    /// Panics if `new_len > self.len()`.
    fn truncate(&mut self, new_len: usize) {
        assert!(new_len <= self.len());

        *self = match *self {
            Self::Single(buf) => Self::Single(&buf[..new_len]),
            Self::Split([a, b]) => {
                if new_len <= a.len() {
                    Self::Single(&a[..new_len])
                } else {
                    Self::Split([a, &b[..new_len - a.len()]])
                }
            }
        }
    }

    /// Copy bytes from within this byte buffer to the provided slice.
    ///
    /// This will also remove those same bytes from the front of this byte
    /// buffer.
    ///
    /// # Panics
    /// Panics if `self.len() < dst.len()`
    fn copy_to_slice(&mut self, dst: &mut [u8]) {
        assert!(self.len() >= dst.len());

        match self {
            Self::Single(buf) => {
                let (head, rest) = buf.split_at(dst.len());
                dst.copy_from_slice(head);
                *buf = rest;
            }
            Self::Split([buf, _]) if buf.len() >= dst.len() => {
                let (head, rest) = buf.split_at(dst.len());
                dst.copy_from_slice(head);
                *buf = rest;
            }
            &mut Self::Split([a, b]) => {
                let (d_head, d_rest) = dst.split_at_mut(a.len());
                let (b_head, b_rest) = b.split_at(d_rest.len());

                d_head.copy_from_slice(a);
                d_rest.copy_from_slice(b_head);
                *self = Self::Single(b_rest);
            }
        }
    }
}

unsafe impl<'a> ParseBuf<'a> for ByteBuffer<'a> {
    fn chunk(&mut self) -> ParseResult<ParseBufChunk<'_, 'a>> {
        match self {
            Self::Single([]) => Err(ParseError::eof()),
            Self::Single(chunk) => Ok(ParseBufChunk::External(chunk)),
            Self::Split([chunk, _]) => Ok(ParseBufChunk::External(chunk)),
        }
    }

    fn advance(&mut self, mut count: usize) {
        match self {
            Self::Single(chunk) => chunk.advance(count),
            Self::Split([chunk, _]) if count < chunk.len() => chunk.advance(count),
            Self::Split([a, b]) => {
                count -= a.len();
                b.advance(count);
                *self = Self::Single(b);
            }
        }
    }
}

macro_rules! assert_same_size {
    ($a:ty, $b:ty) => {{
        if false {
            let _assert_same_size: [u8; ::std::mem::size_of::<$b>()] =
                [0u8; ::std::mem::size_of::<$a>()];
        }
    }};
}

trait Atomic: Sized + Copy {
    type Atomic;

    unsafe fn store(ptr: *const Self, val: Self, order: Ordering);
    unsafe fn load(ptr: *const Self, order: Ordering) -> Self;
}

macro_rules! impl_atomic {
    ($base:ty, $atomic:ty) => {
        impl Atomic for $base {
            type Atomic = $atomic;

            unsafe fn store(ptr: *const Self, val: Self, order: Ordering) {
                assert_same_size!(Self, Self::Atomic);

                let ptr = ptr as *const Self::Atomic;
                (*ptr).store(val, order)
            }

            unsafe fn load(ptr: *const Self, order: Ordering) -> Self {
                assert_same_size!(Self, Self::Atomic);

                let ptr = ptr as *const Self::Atomic;
                (*ptr).load(order)
            }
        }
    };
}

impl_atomic!(u64, std::sync::atomic::AtomicU64);
impl_atomic!(u32, std::sync::atomic::AtomicU32);
impl_atomic!(u16, std::sync::atomic::AtomicU16);
impl_atomic!(i64, std::sync::atomic::AtomicI64);

/// Do an atomic write to the value stored at `ptr`.
///
/// # Safety
/// - `ptr` must be valid for writes.
/// - `ptr` must be properly aligned.
unsafe fn atomic_store<T: Atomic>(ptr: *const T, val: T, order: Ordering) {
    T::store(ptr, val, order)
}

/// Perform an atomic read from the value stored at `ptr`.
///
/// # Safety
/// - `ptr` must be valid for reads.
/// - `ptr` must be properly aligned.
unsafe fn atomic_load<T: Atomic>(ptr: *const T, order: Ordering) -> T {
    T::load(ptr, order)
}

/// Read a performance monitoring counter via the `rdpmc` instruction.
///
/// # Safety
/// - `index` must be a valid PMC index
/// - The current CPU must be allowed to execute the `rdpmc` instruction at the
///   current priviledge level.
///
/// Note that the safety constraints come from the x86 ISA so any violation of
/// them will likely lead to a SIGINT or other such signal.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
unsafe fn rdpmc(index: u32) -> u64 {
    // This saves a few instructions for 64-bit since LLVM doesn't realize
    // that the top 32 bits of RAX:RDX are cleared otherwise.
    #[cfg(target_arch = "x86_64")]
    {
        let lo: u64;
        let hi: u64;

        std::arch::asm!(
            "rdpmc",
            in("ecx") index,
            out("rax") lo,
            out("rdx") hi
        );

        lo | (hi << u32::BITS)
    }

    #[cfg(target_arch = "x86")]
    {
        let lo: u32;
        let hi: u32;

        std::arch::asm!(
            "rdpmc",
            in("ecx") index,
            out("eax") lo,
            out("edx") hi
        );

        (lo as u64) | ((hi as u64) << u32::BITS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buf_copy_over_split() {
        let mut out = [0; 7];
        let mut buf = ByteBuffer::Split([b"aaaaaa", b"bbbbb"]);
        buf.copy_to_slice(&mut out);
        assert_eq!(&out, b"aaaaaab");
        assert_eq!(buf.len(), 4);
    }

    #[test]
    fn buf_copy_to_split() {
        let mut out = [0; 6];
        let mut buf = ByteBuffer::Split([b"aaaaaa", b"bbbbb"]);
        buf.copy_to_slice(&mut out);

        assert_eq!(&out, b"aaaaaa");
        assert_eq!(buf.len(), 5);
    }

    #[test]
    fn buf_copy_before_split() {
        let mut out = [0; 5];
        let mut buf = ByteBuffer::Split([b"aaaaaa", b"bbbbb"]);
        buf.copy_to_slice(&mut out);

        assert_eq!(&out, b"aaaaa");
        assert_eq!(buf.len(), 6);
    }

    #[test]
    fn buf_truncate_over_split() {
        let mut out = [0u8; 11];
        let mut buf = ByteBuffer::Split([b"1234567890", b"abc"]);

        buf.truncate(11);
        assert_eq!(buf.len(), 11);

        buf.copy_to_slice(&mut out);
        assert_eq!(&out, b"1234567890a");
    }

    #[test]
    fn buf_truncate_before_split() {
        let mut out = [0u8; 5];
        let mut buf = ByteBuffer::Split([b"1234567890", b"abc"]);

        buf.truncate(5);
        assert_eq!(buf.len(), 5);

        buf.copy_to_slice(&mut out);
        assert_eq!(&out, b"12345");
    }
}
