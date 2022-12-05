use std::fmt;

use bitflags::bitflags;
use bytes::Buf;
use perf_event_open_sys::bindings::{self, perf_branch_entry};

use crate::samples::{Arm64RegMask, X86RegMask};

use super::{
    BranchSampleType, Parse, ParseBuf, ParseConfig, ReadValue, RecordEvent, SampleRegsAbi,
    SampleType,
};

pub use self::bitflag_defs::*;

/// A sample as gathered by the kernel.
///
/// This struct corresponds to `PERF_RECORD_SAMPLE`. See the [manpage] for more
/// documentation.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Default)]
#[allow(missing_docs)]
#[non_exhaustive]
pub struct Sample {
    pub ip: Option<u64>,
    pub pid: Option<u32>,
    pub tid: Option<u32>,
    pub time: Option<u64>,
    pub addr: Option<u64>,
    pub id: Option<u64>,
    pub stream_id: Option<u64>,
    pub cpu: Option<u32>,
    pub period: Option<u64>,
    pub value: Option<ReadValue>,
    pub callchain: Option<Vec<u64>>,
    pub raw: Option<Vec<u8>>,
    pub lbr_hw_index: Option<u64>,
    pub lbr: Option<Vec<BranchEntry>>,
    pub regs_user: Option<Registers>,
    pub stack_user: Option<Vec<u8>>,
    pub weight: Option<u64>,
    pub data_src: Option<DataSource>,
    pub transaction: Option<Txn>,
    pub regs_intr: Option<Registers>,
    pub phys_addr: Option<u64>,
    pub cgroup: Option<u64>,
    pub data_page_size: Option<u64>,
    pub code_page_size: Option<u64>,
    pub aux: Option<Vec<u8>>,

    /// Extra unparsed bytes at the end of the record.
    ///
    /// This will correspond to new fields not yet supported by the
    /// `perf-event` crate.
    ///
    /// If you're relying on this, please submit a PR to the `perf-event` crate
    /// to add support for whatever new field you are using.
    pub extra: Vec<u8>,
}

/// Describes the captured subset of registers when a sample was taken.
///
/// See the [manpage] for all the details.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Clone, Debug)]
pub struct Registers {
    /// The ABI of the program from which the sample was taken.
    pub abi: SampleRegsAbi,

    /// A bitmask indicating which registers were recorded.
    ///
    /// This is configured as a part of constructing the sampler.
    pub mask: u64,

    /// The recorded values of the registers.
    pub regs: Vec<u64>,
}

/// Describes where in the memory hierarchy the sampled instruction came from.
///
/// See the [manpage] for a full description.
///
/// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct DataSource(u64);

/// Record of a branch taken by the hardware.
#[derive(Copy, Clone, Debug)]
pub struct BranchEntry(perf_branch_entry);

#[allow(missing_docs)]
mod bitflag_defs {
    use super::*;

    bitflags! {
        /// Memory operation.
        ///
        /// This is used by [`DataSource`].
        pub struct MemOp : u64 {
            const NA = bindings::PERF_MEM_OP_NA as _;
            const LOAD = bindings::PERF_MEM_OP_LOAD as _;
            const STORE = bindings::PERF_MEM_OP_STORE as _;
            const PFETCH = bindings::PERF_MEM_OP_PFETCH as _;
            const EXEC = bindings::PERF_MEM_OP_EXEC as _;
        }

        /// Location in the memory hierarchy.
        ///
        /// This is used by [`DataSource`].
        pub struct MemLevel : u64 {
            const NA = bindings::PERF_MEM_LVL_NA as _;
            const HIT = bindings::PERF_MEM_LVL_HIT as _;
            const MISS = bindings::PERF_MEM_LVL_MISS as _;
            const L1 = bindings::PERF_MEM_LVL_L1 as _;
            const LFB = bindings::PERF_MEM_LVL_LFB as _;
            const L2 = bindings::PERF_MEM_LVL_L2 as _;
            const L3 = bindings::PERF_MEM_LVL_L3 as _;
            const LOC_RAM = bindings::PERF_MEM_LVL_LOC_RAM as _;
            const REM_RAM1 = bindings::PERF_MEM_LVL_REM_RAM1 as _;
            const REM_RAM2 = bindings::PERF_MEM_LVL_REM_RAM2 as _;
            const REM_CCE1 = bindings::PERF_MEM_LVL_REM_CCE1 as _;
            const REM_CCE2 = bindings::PERF_MEM_LVL_REM_CCE2 as _;
            const IO = bindings::PERF_MEM_LVL_IO as _;
            const UNC = bindings::PERF_MEM_LVL_UNC as _;
        }

        /// Memory snoop mode.
        ///
        /// This is used by [`DataSource`].
        pub struct MemSnoop : u64 {
            const NA = bindings::PERF_MEM_SNOOP_NA as _;
            const NONE = bindings::PERF_MEM_SNOOP_NONE as _;
            const HIT = bindings::PERF_MEM_SNOOP_HIT as _;
            const MISS = bindings::PERF_MEM_SNOOP_MISS as _;
            const HITM = bindings::PERF_MEM_SNOOP_HITM as _;
        }

        /// Whether the instruction was a locked instruction.
        ///
        /// This is used by [`DataSource`].
        pub struct MemLock : u64 {
            const NA = bindings::PERF_MEM_LOCK_NA as _;
            const LOCKED = bindings::PERF_MEM_LOCK_LOCKED as _;
        }

        /// Memory TLB access.
        ///
        /// This is used by [`DataSource`].
        pub struct MemDtlb : u64 {
            const NA = bindings::PERF_MEM_TLB_NA as _;
            const HIT = bindings::PERF_MEM_TLB_HIT as _;
            const MISS = bindings::PERF_MEM_TLB_MISS as _;
            const L1 = bindings::PERF_MEM_TLB_L1 as _;
            const L2 = bindings::PERF_MEM_TLB_L2 as _;
            const WK = bindings::PERF_MEM_TLB_WK as _;
            const OS = bindings::PERF_MEM_TLB_OS as _;
        }

        /// Extended bits for [`MemSnoop`].
        ///
        /// This is used by [`DataSource`].
        pub struct MemSnoopX : u64 {
            const FWD = bindings::PERF_MEM_SNOOPX_FWD as _;

            // SnoopX is two bits in size but only one field is defined at this time
            #[doc(hidden)]
            const _MASK = 0x3;
        }

        /// Info about a transactional memory event.
        pub struct Txn: u64 {
            const ELISION = bindings::PERF_TXN_ELISION as _;
            const TRANSACTION = bindings::PERF_TXN_TRANSACTION as _;
            const SYNC = bindings::PERF_TXN_SYNC as _;
            const ASYNC = bindings::PERF_TXN_ASYNC as _;
            const RETRY = bindings::PERF_TXN_RETRY as _;
            const CONFLICT = bindings::PERF_TXN_CONFLICT as _;
            const CAPACITY_WRITE = bindings::PERF_TXN_CAPACITY_WRITE as _;
            const CAPACITY_READ = bindings::PERF_TXN_CAPACITY_READ as _;

            const ABORT_MASK = bindings::PERF_TXN_ABORT_MASK as _;
        }
    }

    impl Txn {
        /// Create a new Txn from the raw bitfield value.
        pub const fn new(bits: u64) -> Self {
            Self { bits }
        }

        /// A user-specified abort code.
        pub fn abort(&self) -> u32 {
            (self.bits() >> bindings::PERF_TXN_ABORT_SHIFT) as _
        }
    }
}

enum_binding! {
    /// Memory hierarchy level number.
    ///
    /// This is a field within [`DataSource`]. It is not documented in the [manpage]
    /// but is present within the perf_event headers.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub struct MemLevelNum : u8 {
        const L1 = bindings::PERF_MEM_LVLNUM_L1 as _;
        const L2 = bindings::PERF_MEM_LVLNUM_L2 as _;
        const L3 = bindings::PERF_MEM_LVLNUM_L3 as _;
        const L4 = bindings::PERF_MEM_LVLNUM_L4 as _;

        const ANY_CACHE = bindings::PERF_MEM_LVLNUM_ANY_CACHE as _;
        const LFB = bindings::PERF_MEM_LVLNUM_LFB as _;
        const RAM = bindings::PERF_MEM_LVLNUM_RAM as _;
        const PMEM = bindings::PERF_MEM_LVLNUM_PMEM as _;
        const NA = bindings::PERF_MEM_LVLNUM_NA as _;
    }
}

enum_binding! {
    /// Branch type as used by the last branch record.
    ///
    /// This is a field present within [`BranchEntry`]. It is not documented in the
    /// [manpage] but is present within the perf_event headers.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub struct BranchType : u8 {
        const UNKNOWN = bindings::PERF_BR_UNKNOWN as _;
        const COND = bindings::PERF_BR_COND as _;
        const UNCOND = bindings::PERF_BR_UNCOND as _;
        const IND = bindings::PERF_BR_IND as _;
        const CALL = bindings::PERF_BR_CALL as _;
        const IND_CALL = bindings::PERF_BR_IND_CALL as _;
        const RET = bindings::PERF_BR_RET as _;
        const SYSCALL = bindings::PERF_BR_SYSCALL as _;
        const COND_CALL = bindings::PERF_BR_COND_CALL as _;
        const COND_RET = bindings::PERF_BR_COND_RET as _;
    }

}

impl DataSource {
    /// Type of opcode.
    pub fn mem_op(&self) -> MemOp {
        MemOp::from_bits_truncate(self.0)
    }

    /// Memory hierarchy level hit or miss.
    pub fn mem_lvl(&self) -> MemLevel {
        MemLevel::from_bits_truncate(self.0 >> bindings::PERF_MEM_LVL_SHIFT)
    }

    /// Snoop mode.
    pub fn mem_snoop(&self) -> MemSnoop {
        MemSnoop::from_bits_truncate(self.0 >> bindings::PERF_MEM_SNOOP_SHIFT)
    }

    /// Lock instruction.
    pub fn mem_lock(&self) -> MemLock {
        MemLock::from_bits_truncate(self.0 >> bindings::PERF_MEM_LOCK_SHIFT)
    }

    /// TLB access hit or miss.
    pub fn mem_dtlb(&self) -> MemDtlb {
        MemDtlb::from_bits_truncate(self.0 >> bindings::PERF_MEM_TLB_SHIFT)
    }

    /// Memory hierarchy level number.
    ///
    /// This field is not documented in the [manpage] but is present within the
    /// kernel headers.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub fn mem_lvl_num(&self) -> MemLevelNum {
        MemLevelNum(((self.0 >> bindings::PERF_MEM_LVLNUM_SHIFT) & 0xF) as _)
    }

    /// Whether the memory access was remote.
    ///
    /// This field is not documented in the [manpage] but is present within the
    /// kernel headers.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub fn mem_remote(&self) -> bool {
        ((self.0 >> bindings::PERF_MEM_REMOTE_SHIFT) & 0x1) != 0
    }

    /// Snoop mode, extended.
    ///
    /// This field is not documented in the [manpage] but is present within the
    /// kernel headers.
    ///
    /// [manpage]: http://man7.org/linux/man-pages/man2/perf_event_open.2.html
    pub fn mem_snoopx(&self) -> MemSnoopX {
        MemSnoopX::from_bits_truncate(self.0 >> bindings::PERF_MEM_SNOOPX_SHIFT)
    }
}

impl BranchEntry {
    /// Address of the source instruction.
    ///
    /// This may not always be a branch instruction.
    pub fn from(&self) -> u64 {
        self.0.from
    }

    /// Address of the branch target.
    pub fn to(&self) -> u64 {
        self.0.to
    }

    /// Whether the branch was mispredicted.
    pub fn mispred(&self) -> bool {
        self.0.mispred() != 0
    }

    /// Whether the branch was predicted correctly.
    pub fn predicted(&self) -> bool {
        self.0.predicted() != 0
    }

    /// Whether the branch occurred within a transaction.
    pub fn in_tx(&self) -> bool {
        self.0.in_tx() != 0
    }

    /// Whether the branch was due to a transaction abort.
    pub fn abort(&self) -> bool {
        self.0.abort() != 0
    }

    /// The cycle count since the last branch.
    pub fn cycles(&self) -> u16 {
        self.0.cycles() as _
    }

    /// Branch type.
    ///
    /// This field is not documented within the manpage but is present within
    /// the perf_event headers.
    pub fn ty(&self) -> BranchType {
        BranchType(self.0.type_() as _)
    }
}

macro_rules! decl_reg_accessors {
    {
        $(
            $( #[$attr:meta] )*
            $method:ident = $mask:expr ;
        )*
    } => {
        impl Registers {
            $(
                $( #[$attr] )*
                #[allow(missing_docs)]
                pub fn $method(&self) -> Option<u64> {
                    const MASK: u64 = $mask.bits();
                    // Mask with all bits before MASK set
                    const LEADING_MASK: u64 = (1 << MASK.trailing_zeros()) - 1;

                    if self.mask & MASK != 0 {
                        let index = (self.mask & LEADING_MASK).count_ones();
                        Some(self.regs[index as usize])
                    } else {
                        None
                    }
                }
            )*
        }
    }
}

macro_rules! decl_xmm_reg_accessors {
    {
        $(
            $( #[$attr:meta] )*
            $method:ident = $mask:expr ;
        )*
    } => {
        impl Registers {
            $(
                $( #[$attr] )*
                #[allow(missing_docs)]
                pub fn $method(&self) -> Option<u128> {
                    const MASK: u64 = $mask.bits();
                    // Mask with all bits before MASK set
                    const LEADING_MASK: u64 = (1 << MASK.trailing_zeros()) - 1;

                    if self.mask & MASK == MASK {
                        let index = (self.mask & LEADING_MASK).count_ones() as usize;

                        let lo = self.regs[index];
                        let hi = self.regs[index + 1];
                        Some(((hi as u128) << 64) | (lo as u128))
                    } else {
                        None
                    }
                }
            )*
        }
    }
}

decl_reg_accessors! {
    x86_ax = X86RegMask::AX;
    x86_bx = X86RegMask::BX;
    x86_cx = X86RegMask::CX;
    x86_dx = X86RegMask::DX;
    x86_si = X86RegMask::SI;
    x86_di = X86RegMask::DI;
    x86_bp = X86RegMask::BP;
    x86_sp = X86RegMask::SP;
    x86_ip = X86RegMask::IP;
    x86_flags = X86RegMask::FLAGS;
    x86_cs = X86RegMask::CS;
    x86_ss = X86RegMask::SS;
    x86_ds = X86RegMask::DS;
    x86_es = X86RegMask::ES;
    x86_fs = X86RegMask::FS;
    x86_gs = X86RegMask::GS;
    x86_r8 = X86RegMask::R8;
    x86_r9 = X86RegMask::R9;
    x86_r10 = X86RegMask::R10;
    x86_r11 = X86RegMask::R11;
    x86_r12 = X86RegMask::R12;
    x86_r13 = X86RegMask::R13;
    x86_r14 = X86RegMask::R14;
    x86_r15 = X86RegMask::R15;

    // XMM registers are handled separately since they are 128 bits

    x86_eax = X86RegMask::EAX;
    x86_ebx = X86RegMask::EBX;
    x86_ecx = X86RegMask::ECX;
    x86_edx = X86RegMask::EDX;
    x86_esi = X86RegMask::ESI;
    x86_edi = X86RegMask::EDI;
    x86_ebp = X86RegMask::EBP;
    x86_esp = X86RegMask::ESP;
    x86_eip = X86RegMask::EIP;

    x86_rax = X86RegMask::RAX;
    x86_rbx = X86RegMask::RBX;
    x86_rcx = X86RegMask::RCX;
    x86_rdx = X86RegMask::RDX;
    x86_rsi = X86RegMask::RSI;
    x86_rdi = X86RegMask::RDI;
    x86_rbp = X86RegMask::RBP;
    x86_rsp = X86RegMask::RSP;
    x86_rip = X86RegMask::RIP;
}

decl_xmm_reg_accessors! {
    x86_xmm0 = X86RegMask::XMM0;
    x86_xmm1 = X86RegMask::XMM1;
    x86_xmm2 = X86RegMask::XMM2;
    x86_xmm3 = X86RegMask::XMM3;
    x86_xmm4 = X86RegMask::XMM4;
    x86_xmm5 = X86RegMask::XMM5;
    x86_xmm6 = X86RegMask::XMM6;
    x86_xmm7 = X86RegMask::XMM7;
    x86_xmm8 = X86RegMask::XMM8;
    x86_xmm9 = X86RegMask::XMM9;
    x86_xmm10 = X86RegMask::XMM10;
    x86_xmm11 = X86RegMask::XMM11;
    x86_xmm12 = X86RegMask::XMM12;
    x86_xmm13 = X86RegMask::XMM13;
    x86_xmm14 = X86RegMask::XMM14;
    x86_xmm15 = X86RegMask::XMM15;
}

decl_reg_accessors! {
    arm64_x0 = Arm64RegMask::X0;
    arm64_x1 = Arm64RegMask::X1;
    arm64_x2 = Arm64RegMask::X2;
    arm64_x3 = Arm64RegMask::X3;
    arm64_x4 = Arm64RegMask::X4;
    arm64_x5 = Arm64RegMask::X5;
    arm64_x6 = Arm64RegMask::X6;
    arm64_x7 = Arm64RegMask::X7;
    arm64_x8 = Arm64RegMask::X8;
    arm64_x9 = Arm64RegMask::X9;
    arm64_x10 = Arm64RegMask::X10;
    arm64_x11 = Arm64RegMask::X11;
    arm64_x12 = Arm64RegMask::X12;
    arm64_x13 = Arm64RegMask::X13;
    arm64_x14 = Arm64RegMask::X14;
    arm64_x15 = Arm64RegMask::X15;
    arm64_x16 = Arm64RegMask::X16;
    arm64_x17 = Arm64RegMask::X17;
    arm64_x18 = Arm64RegMask::X18;
    arm64_x19 = Arm64RegMask::X19;
    arm64_x20 = Arm64RegMask::X20;
    arm64_x21 = Arm64RegMask::X21;
    arm64_x22 = Arm64RegMask::X22;
    arm64_x23 = Arm64RegMask::X23;
    arm64_x24 = Arm64RegMask::X24;
    arm64_x25 = Arm64RegMask::X25;
    arm64_x26 = Arm64RegMask::X26;
    arm64_x27 = Arm64RegMask::X27;
    arm64_x28 = Arm64RegMask::X28;
    arm64_x29 = Arm64RegMask::X29;
    arm64_lr  = Arm64RegMask::LR;
    arm64_sp  = Arm64RegMask::SP;
    arm64_pc  = Arm64RegMask::PC;
}

impl Registers {
    fn parse_regs<B: Buf>(mut mask: u64, buf: &mut B) -> Self {
        let abi = buf.get_u64_ne();

        // If the ABI is NONE then the kernel doesn't output anything. See
        // the kernel source link below to confirm.
        // https://sourcegraph.com/github.com/torvalds/linux@b7b275e60bcd5f89771e865a8239325f86d9927d/-/blob/kernel/events/core.c?L7184
        if abi == SampleRegsAbi::NONE.0 {
            mask = 0;
        }

        let regs = std::iter::repeat_with(|| buf.get_u64_ne())
            .take(mask.count_ones() as _)
            .collect();

        Self {
            abi: abi.into(),
            mask,
            regs,
        }
    }
}

impl Parse for DataSource {
    fn parse<B: Buf>(_: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self(buf.get_u64_ne())
    }
}

impl Parse for BranchEntry {
    fn parse<B: Buf>(_: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        Self(unsafe { buf.parse_transmute() })
    }
}

impl Parse for Sample {
    // The order of fields here should match the order they are emitted within
    // the kernel. See the source code at the link below to verify.
    //
    // https://sourcegraph.com/github.com/torvalds/linux@b7b275e60bcd5f89771e865a8239325f86d9927d/-/blob/kernel/events/core.c?L7052
    fn parse<B: Buf>(config: &ParseConfig, buf: &mut B) -> Self
    where
        Self: Sized,
    {
        let sty = config.sample_type;

        let sample_id = sty
            .contains(SampleType::IDENTIFIER)
            .then(|| buf.get_u64_ne());
        let ip = sty.contains(SampleType::IP).then(|| buf.get_u64_ne());
        let pid = sty.contains(SampleType::TID).then(|| buf.get_u32_ne());
        let tid = sty.contains(SampleType::TID).then(|| buf.get_u32_ne());
        let time = sty.contains(SampleType::TIME).then(|| buf.get_u64_ne());
        let addr = sty.contains(SampleType::ADDR).then(|| buf.get_u64_ne());
        let id = sty.contains(SampleType::ID).then(|| buf.get_u64_ne());
        let stream_id = sty
            .contains(SampleType::STREAM_ID)
            .then(|| buf.get_u64_ne());
        let cpu = sty.contains(SampleType::CPU).then(|| {
            let cpu = buf.get_u32_ne();
            buf.get_u32_ne(); // res
            cpu
        });
        let period = sty.contains(SampleType::PERIOD).then(|| buf.get_u64_ne());
        let value = sty
            .contains(SampleType::READ)
            .then(|| ReadValue::parse(config, buf));
        let callchain = sty.contains(SampleType::CALLCHAIN).then(|| {
            let len = buf.get_u64_ne() as usize;
            std::iter::repeat_with(|| buf.get_u64_ne())
                .take(len)
                .collect()
        });
        let raw = sty.contains(SampleType::RAW).then(|| {
            let len = buf.get_u64_ne() as usize;
            buf.parse_vec(len)
        });
        let (lbr, lbr_hw_index) = if sty.contains(SampleType::BRANCH_STACK) {
            let len = buf.get_u64_ne() as usize;
            let hw_index = config
                .branch_sample_type
                .contains(BranchSampleType::HW_INDEX)
                .then(|| buf.get_u64_ne());
            let lbr = std::iter::repeat_with(|| BranchEntry::parse(config, buf))
                .take(len)
                .collect();
            (Some(lbr), hw_index)
        } else {
            (None, None)
        };
        let regs_user = sty
            .contains(SampleType::REGS_USER)
            .then(|| Registers::parse_regs(config.regs_user, buf));
        let stack_user = sty.contains(SampleType::STACK_USER).then(|| {
            let len = buf.get_u64_ne() as usize;
            let mut data = buf.parse_vec(len);

            if len != 0 {
                let dyn_len = buf.get_u64_ne() as usize;
                data.truncate(dyn_len);
            }

            data
        });
        let weight = (sty.contains(SampleType::WEIGHT) || sty.contains(SampleType::WEIGHT_STRUCT))
            .then(|| buf.get_u64_ne());
        let data_src = sty
            .contains(SampleType::DATA_SRC)
            .then(|| DataSource::parse(config, buf));
        let transaction = sty
            .contains(SampleType::TRANSACTION)
            .then(|| Txn::new(buf.get_u64_ne()));
        let regs_intr = sty
            .contains(SampleType::REGS_INTR)
            .then(|| Registers::parse_regs(config.regs_intr, buf));
        let phys_addr = sty
            .contains(SampleType::PHYS_ADDR)
            .then(|| buf.get_u64_ne());
        let cgroup = sty.contains(SampleType::CGROUP).then(|| buf.get_u64_ne());
        let data_page_size = sty
            .contains(SampleType::DATA_PAGE_SIZE)
            .then(|| buf.get_u64_ne());
        let code_page_size = sty
            .contains(SampleType::CODE_PAGE_SIZE)
            .then(|| buf.get_u64_ne());
        let aux = sty.contains(SampleType::AUX).then(|| {
            let len = buf.get_u64_ne() as usize;
            buf.parse_vec(len)
        });

        Self {
            ip,
            pid,
            tid,
            time,
            addr,
            id: id.or(sample_id),
            stream_id,
            cpu,
            period,
            value,
            callchain,
            raw,
            lbr,
            lbr_hw_index,
            regs_user,
            stack_user,
            weight,
            data_src,
            transaction,
            regs_intr,
            phys_addr,
            aux,
            cgroup,
            data_page_size,
            code_page_size,

            extra: buf.parse_remainder(),
        }
    }
}

impl fmt::Debug for DataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataSource")
            .field("mem_op", &self.mem_op())
            .field("mem_lvl", &self.mem_lvl())
            .field("mem_snoop", &self.mem_snoop())
            .field("mem_lock", &self.mem_lock())
            .field("mem_dtlb", &self.mem_dtlb())
            .field("mem_lvl_num", &self.mem_lvl_num())
            .field("mem_remote", &self.mem_remote())
            .field("mem_snoopx", &self.mem_snoopx())
            .finish()
    }
}

impl From<Sample> for RecordEvent {
    fn from(sample: Sample) -> Self {
        Self::Sample(sample)
    }
}

// Sample has many fields and most of the time only a few of them will be
// present.
//
// Showing all the None options would make the debug output much less useful so
// instead we override the debug impl with one that only shows the present
// fields.
impl fmt::Debug for Sample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Destructure so that new fields cause a compilation error.
        //
        // If you're adding a new field then all you need to do is
        // - add it to the list of fields below this comment
        // - add a new dbg_field!(dbg, <your field name>) at the end of the
        //   list.
        let Sample {
            ip,
            pid,
            tid,
            time,
            addr,
            id,
            stream_id,
            cpu,
            period,
            value,
            callchain,
            raw,
            lbr,
            lbr_hw_index,
            regs_user,
            stack_user,
            weight,
            data_src,
            transaction,
            regs_intr,
            phys_addr,
            cgroup,
            data_page_size,
            code_page_size,
            aux,
            extra,
        } = self;

        let mut dbg = f.debug_struct("Sample");

        macro_rules! dbg_field {
            ($dbg:expr, $field:ident) => {
                if let Some(value) = $field {
                    $dbg.field(stringify!($field), value);
                }
            };
        }

        // Some fields are actually addresses and it makes sense to format
        // these in hex instead of decimal.
        struct Hex<T>(T);
        impl<T: fmt::UpperHex> fmt::Debug for Hex<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        if let Some(ip) = ip {
            dbg.field("ip", &Hex(ip));
        }

        dbg_field!(dbg, pid);
        dbg_field!(dbg, tid);
        dbg_field!(dbg, time);

        if let Some(addr) = addr {
            dbg.field("addr", &Hex(addr));
        }

        dbg_field!(dbg, id);
        dbg_field!(dbg, stream_id);
        dbg_field!(dbg, cpu);
        dbg_field!(dbg, period);
        dbg_field!(dbg, value);
        dbg_field!(dbg, callchain);
        dbg_field!(dbg, raw);
        dbg_field!(dbg, lbr_hw_index);
        dbg_field!(dbg, lbr);
        dbg_field!(dbg, regs_user);
        dbg_field!(dbg, stack_user);
        dbg_field!(dbg, weight);
        dbg_field!(dbg, data_src);
        dbg_field!(dbg, transaction);
        dbg_field!(dbg, regs_intr);

        if let Some(phys_addr) = phys_addr {
            dbg.field("phys_addr", &Hex(phys_addr));
        }

        dbg_field!(dbg, cgroup);
        dbg_field!(dbg, data_page_size);
        dbg_field!(dbg, code_page_size);
        dbg_field!(dbg, aux);

        if !extra.is_empty() {
            dbg.field("extra", extra);
        }

        dbg.finish_non_exhaustive()
    }
}
