#![allow(missing_docs)]

use bitflags::bitflags;
use perf_event_open_sys::bindings;

/// Trait for types which can be used as a register mask.
pub trait RegMask {
    fn as_bits(&self) -> u64;
}

impl RegMask for u64 {
    fn as_bits(&self) -> u64 {
        *self
    }
}

bitflags! {
    /// Register mask for Arm64 registers.
    pub struct Arm64RegMask : u64 {
        const X0  = 1 << bindings::PERF_REG_ARM64_X0;
        const X1  = 1 << bindings::PERF_REG_ARM64_X1;
        const X2  = 1 << bindings::PERF_REG_ARM64_X2;
        const X3  = 1 << bindings::PERF_REG_ARM64_X3;
        const X4  = 1 << bindings::PERF_REG_ARM64_X4;
        const X5  = 1 << bindings::PERF_REG_ARM64_X5;
        const X6  = 1 << bindings::PERF_REG_ARM64_X6;
        const X7  = 1 << bindings::PERF_REG_ARM64_X7;
        const X8  = 1 << bindings::PERF_REG_ARM64_X8;
        const X9  = 1 << bindings::PERF_REG_ARM64_X9;
        const X10 = 1 << bindings::PERF_REG_ARM64_X10;
        const X11 = 1 << bindings::PERF_REG_ARM64_X11;
        const X12 = 1 << bindings::PERF_REG_ARM64_X12;
        const X13 = 1 << bindings::PERF_REG_ARM64_X13;
        const X14 = 1 << bindings::PERF_REG_ARM64_X14;
        const X15 = 1 << bindings::PERF_REG_ARM64_X15;
        const X16 = 1 << bindings::PERF_REG_ARM64_X16;
        const X17 = 1 << bindings::PERF_REG_ARM64_X17;
        const X18 = 1 << bindings::PERF_REG_ARM64_X18;
        const X19 = 1 << bindings::PERF_REG_ARM64_X19;
        const X20 = 1 << bindings::PERF_REG_ARM64_X20;
        const X21 = 1 << bindings::PERF_REG_ARM64_X21;
        const X22 = 1 << bindings::PERF_REG_ARM64_X22;
        const X23 = 1 << bindings::PERF_REG_ARM64_X23;
        const X24 = 1 << bindings::PERF_REG_ARM64_X24;
        const X25 = 1 << bindings::PERF_REG_ARM64_X25;
        const X26 = 1 << bindings::PERF_REG_ARM64_X26;
        const X27 = 1 << bindings::PERF_REG_ARM64_X27;
        const X28 = 1 << bindings::PERF_REG_ARM64_X28;
        const X29 = 1 << bindings::PERF_REG_ARM64_X29;
        const LR  = 1 << bindings::PERF_REG_ARM64_LR;
        const SP  = 1 << bindings::PERF_REG_ARM64_SP;
        const PC  = 1 << bindings::PERF_REG_ARM64_PC;
    }
}

impl RegMask for Arm64RegMask {
    fn as_bits(&self) -> u64 {
        self.bits()
    }
}

bitflags! {
    /// Register masks for ARM.
    pub struct ArmRegMask : u64 {
        const R0 = 1 << bindings::PERF_REG_ARM_R0;
        const R1 = 1 << bindings::PERF_REG_ARM_R1;
        const R2 = 1 << bindings::PERF_REG_ARM_R2;
        const R3 = 1 << bindings::PERF_REG_ARM_R3;
        const R4 = 1 << bindings::PERF_REG_ARM_R4;
        const R5 = 1 << bindings::PERF_REG_ARM_R5;
        const R6 = 1 << bindings::PERF_REG_ARM_R6;
        const R7 = 1 << bindings::PERF_REG_ARM_R7;
        const R8 = 1 << bindings::PERF_REG_ARM_R8;
        const R9 = 1 << bindings::PERF_REG_ARM_R9;
        const R10 = 1 << bindings::PERF_REG_ARM_R10;
        const FP = 1 << bindings::PERF_REG_ARM_FP;
        const IP = 1 << bindings::PERF_REG_ARM_IP;
        const SP = 1 << bindings::PERF_REG_ARM_SP;
        const LR = 1 << bindings::PERF_REG_ARM_LR;
        const PC = 1 << bindings::PERF_REG_ARM_PC;
    }
}

impl RegMask for ArmRegMask {
    fn as_bits(&self) -> u64 {
        self.bits()
    }
}

bitflags! {
    /// Register masks for C-SKY.
    pub struct CSkyRegMask : u64 {
        const TLS = 1 << bindings::PERF_REG_CSKY_TLS;
        const LR = 1 << bindings::PERF_REG_CSKY_LR;
        const PC = 1 << bindings::PERF_REG_CSKY_PC;
        const SR = 1 << bindings::PERF_REG_CSKY_SR;
        const SP = 1 << bindings::PERF_REG_CSKY_SP;
        const ORIG_A0 = 1 << bindings::PERF_REG_CSKY_ORIG_A0;
        const A0 = 1 << bindings::PERF_REG_CSKY_A0;
        const A1 = 1 << bindings::PERF_REG_CSKY_A1;
        const A2 = 1 << bindings::PERF_REG_CSKY_A2;
        const A3 = 1 << bindings::PERF_REG_CSKY_A3;
        const REGS0 = 1 << bindings::PERF_REG_CSKY_REGS0;
        const REGS1 = 1 << bindings::PERF_REG_CSKY_REGS1;
        const REGS2 = 1 << bindings::PERF_REG_CSKY_REGS2;
        const REGS3 = 1 << bindings::PERF_REG_CSKY_REGS3;
        const REGS4 = 1 << bindings::PERF_REG_CSKY_REGS4;
        const REGS5 = 1 << bindings::PERF_REG_CSKY_REGS5;
        const REGS6 = 1 << bindings::PERF_REG_CSKY_REGS6;
        const REGS7 = 1 << bindings::PERF_REG_CSKY_REGS7;
        const REGS8 = 1 << bindings::PERF_REG_CSKY_REGS8;
        const REGS9 = 1 << bindings::PERF_REG_CSKY_REGS9;
    }
}

impl RegMask for CSkyRegMask {
    fn as_bits(&self) -> u64 {
        self.bits()
    }
}

bitflags! {
    /// Register masks for MIPS.
    pub struct MipsRegMask : u64 {
        const PC = 1 << bindings::PERF_REG_MIPS_PC;
        const R1 = 1 << bindings::PERF_REG_MIPS_R1;
        const R2 = 1 << bindings::PERF_REG_MIPS_R2;
        const R3 = 1 << bindings::PERF_REG_MIPS_R3;
        const R4 = 1 << bindings::PERF_REG_MIPS_R4;
        const R5 = 1 << bindings::PERF_REG_MIPS_R5;
        const R6 = 1 << bindings::PERF_REG_MIPS_R6;
        const R7 = 1 << bindings::PERF_REG_MIPS_R7;
        const R8 = 1 << bindings::PERF_REG_MIPS_R8;
        const R9 = 1 << bindings::PERF_REG_MIPS_R9;
        const R10 = 1 << bindings::PERF_REG_MIPS_R10;
        const R11 = 1 << bindings::PERF_REG_MIPS_R11;
        const R12 = 1 << bindings::PERF_REG_MIPS_R12;
        const R13 = 1 << bindings::PERF_REG_MIPS_R13;
        const R14 = 1 << bindings::PERF_REG_MIPS_R14;
        const R15 = 1 << bindings::PERF_REG_MIPS_R15;
        const R16 = 1 << bindings::PERF_REG_MIPS_R16;
        const R17 = 1 << bindings::PERF_REG_MIPS_R17;
        const R18 = 1 << bindings::PERF_REG_MIPS_R18;
        const R19 = 1 << bindings::PERF_REG_MIPS_R19;
        const R20 = 1 << bindings::PERF_REG_MIPS_R20;
        const R21 = 1 << bindings::PERF_REG_MIPS_R21;
        const R22 = 1 << bindings::PERF_REG_MIPS_R22;
        const R23 = 1 << bindings::PERF_REG_MIPS_R23;
        const R24 = 1 << bindings::PERF_REG_MIPS_R24;
        const R25 = 1 << bindings::PERF_REG_MIPS_R25;
        const R26 = 1 << bindings::PERF_REG_MIPS_R26;
        const R27 = 1 << bindings::PERF_REG_MIPS_R27;
        const R28 = 1 << bindings::PERF_REG_MIPS_R28;
        const R29 = 1 << bindings::PERF_REG_MIPS_R29;
        const R30 = 1 << bindings::PERF_REG_MIPS_R30;
        const R31 = 1 << bindings::PERF_REG_MIPS_R31;
    }
}

impl RegMask for MipsRegMask {
    fn as_bits(&self) -> u64 {
        self.bits()
    }
}

bitflags! {
    /// Register masks for PowerPC.
    pub struct PowerPCRegMask : u64 {
        const R0 = 1 << bindings::PERF_REG_POWERPC_R0;
        const R1 = 1 << bindings::PERF_REG_POWERPC_R1;
        const R2 = 1 << bindings::PERF_REG_POWERPC_R2;
        const R3 = 1 << bindings::PERF_REG_POWERPC_R3;
        const R4 = 1 << bindings::PERF_REG_POWERPC_R4;
        const R5 = 1 << bindings::PERF_REG_POWERPC_R5;
        const R6 = 1 << bindings::PERF_REG_POWERPC_R6;
        const R7 = 1 << bindings::PERF_REG_POWERPC_R7;
        const R8 = 1 << bindings::PERF_REG_POWERPC_R8;
        const R9 = 1 << bindings::PERF_REG_POWERPC_R9;
        const R10 = 1 << bindings::PERF_REG_POWERPC_R10;
        const R11 = 1 << bindings::PERF_REG_POWERPC_R11;
        const R12 = 1 << bindings::PERF_REG_POWERPC_R12;
        const R13 = 1 << bindings::PERF_REG_POWERPC_R13;
        const R14 = 1 << bindings::PERF_REG_POWERPC_R14;
        const R15 = 1 << bindings::PERF_REG_POWERPC_R15;
        const R16 = 1 << bindings::PERF_REG_POWERPC_R16;
        const R17 = 1 << bindings::PERF_REG_POWERPC_R17;
        const R18 = 1 << bindings::PERF_REG_POWERPC_R18;
        const R19 = 1 << bindings::PERF_REG_POWERPC_R19;
        const R20 = 1 << bindings::PERF_REG_POWERPC_R20;
        const R21 = 1 << bindings::PERF_REG_POWERPC_R21;
        const R22 = 1 << bindings::PERF_REG_POWERPC_R22;
        const R23 = 1 << bindings::PERF_REG_POWERPC_R23;
        const R24 = 1 << bindings::PERF_REG_POWERPC_R24;
        const R25 = 1 << bindings::PERF_REG_POWERPC_R25;
        const R26 = 1 << bindings::PERF_REG_POWERPC_R26;
        const R27 = 1 << bindings::PERF_REG_POWERPC_R27;
        const R28 = 1 << bindings::PERF_REG_POWERPC_R28;
        const R29 = 1 << bindings::PERF_REG_POWERPC_R29;
        const R30 = 1 << bindings::PERF_REG_POWERPC_R30;
        const R31 = 1 << bindings::PERF_REG_POWERPC_R31;
        const NIP = 1 << bindings::PERF_REG_POWERPC_NIP;
        const MSR = 1 << bindings::PERF_REG_POWERPC_MSR;
        const ORIG_R3 = 1 << bindings::PERF_REG_POWERPC_ORIG_R3;
        const CTR = 1 << bindings::PERF_REG_POWERPC_CTR;
        const LINK = 1 << bindings::PERF_REG_POWERPC_LINK;
        const XER = 1 << bindings::PERF_REG_POWERPC_XER;
        const CCR = 1 << bindings::PERF_REG_POWERPC_CCR;
        const SOFTE = 1 << bindings::PERF_REG_POWERPC_SOFTE;
        const TRAP = 1 << bindings::PERF_REG_POWERPC_TRAP;
        const DAR = 1 << bindings::PERF_REG_POWERPC_DAR;
        const DSISR = 1 << bindings::PERF_REG_POWERPC_DSISR;
        const SIER = 1 << bindings::PERF_REG_POWERPC_SIER;
        const MMCRA = 1 << bindings::PERF_REG_POWERPC_MMCRA;
        const MMCR0 = 1 << bindings::PERF_REG_POWERPC_MMCR0;
        const MMCR1 = 1 << bindings::PERF_REG_POWERPC_MMCR1;
        const MMCR2 = 1 << bindings::PERF_REG_POWERPC_MMCR2;
        const MMCR3 = 1 << bindings::PERF_REG_POWERPC_MMCR3;
        const SIER2 = 1 << bindings::PERF_REG_POWERPC_SIER2;
        const SIER3 = 1 << bindings::PERF_REG_POWERPC_SIER3;
        const PMC1 = 1 << bindings::PERF_REG_POWERPC_PMC1;
        const PMC2 = 1 << bindings::PERF_REG_POWERPC_PMC2;
        const PMC3 = 1 << bindings::PERF_REG_POWERPC_PMC3;
        const PMC4 = 1 << bindings::PERF_REG_POWERPC_PMC4;
        const PMC5 = 1 << bindings::PERF_REG_POWERPC_PMC5;
        const PMC6 = 1 << bindings::PERF_REG_POWERPC_PMC6;
        const SDAR = 1 << bindings::PERF_REG_POWERPC_SDAR;
        const SIAR = 1 << bindings::PERF_REG_POWERPC_SIAR;
    }
}

impl RegMask for PowerPCRegMask {
    fn as_bits(&self) -> u64 {
        self.bits()
    }
}

bitflags! {
    /// Register masks for RISC-V.
    pub struct RiscvRegMask : u64 {
        const PC = 1 << bindings::PERF_REG_RISCV_PC;
        const RA = 1 << bindings::PERF_REG_RISCV_RA;
        const SP = 1 << bindings::PERF_REG_RISCV_SP;
        const GP = 1 << bindings::PERF_REG_RISCV_GP;
        const TP = 1 << bindings::PERF_REG_RISCV_TP;
        const T0 = 1 << bindings::PERF_REG_RISCV_T0;
        const T1 = 1 << bindings::PERF_REG_RISCV_T1;
        const T2 = 1 << bindings::PERF_REG_RISCV_T2;
        const S0 = 1 << bindings::PERF_REG_RISCV_S0;
        const S1 = 1 << bindings::PERF_REG_RISCV_S1;
        const A0 = 1 << bindings::PERF_REG_RISCV_A0;
        const A1 = 1 << bindings::PERF_REG_RISCV_A1;
        const A2 = 1 << bindings::PERF_REG_RISCV_A2;
        const A3 = 1 << bindings::PERF_REG_RISCV_A3;
        const A4 = 1 << bindings::PERF_REG_RISCV_A4;
        const A5 = 1 << bindings::PERF_REG_RISCV_A5;
        const A6 = 1 << bindings::PERF_REG_RISCV_A6;
        const A7 = 1 << bindings::PERF_REG_RISCV_A7;
        const S2 = 1 << bindings::PERF_REG_RISCV_S2;
        const S3 = 1 << bindings::PERF_REG_RISCV_S3;
        const S4 = 1 << bindings::PERF_REG_RISCV_S4;
        const S5 = 1 << bindings::PERF_REG_RISCV_S5;
        const S6 = 1 << bindings::PERF_REG_RISCV_S6;
        const S7 = 1 << bindings::PERF_REG_RISCV_S7;
        const S8 = 1 << bindings::PERF_REG_RISCV_S8;
        const S9 = 1 << bindings::PERF_REG_RISCV_S9;
        const S10 = 1 << bindings::PERF_REG_RISCV_S10;
        const S11 = 1 << bindings::PERF_REG_RISCV_S11;
        const T3 = 1 << bindings::PERF_REG_RISCV_T3;
        const T4 = 1 << bindings::PERF_REG_RISCV_T4;
        const T5 = 1 << bindings::PERF_REG_RISCV_T5;
        const T6 = 1 << bindings::PERF_REG_RISCV_T6;
    }
}

impl RegMask for RiscvRegMask {
    fn as_bits(&self) -> u64 {
        self.bits()
    }
}

bitflags! {
    /// Register masks for S390.
    pub struct S390RegMask : u64 {
        const R0 = 1 << bindings::PERF_REG_S390_R0;
        const R1 = 1 << bindings::PERF_REG_S390_R1;
        const R2 = 1 << bindings::PERF_REG_S390_R2;
        const R3 = 1 << bindings::PERF_REG_S390_R3;
        const R4 = 1 << bindings::PERF_REG_S390_R4;
        const R5 = 1 << bindings::PERF_REG_S390_R5;
        const R6 = 1 << bindings::PERF_REG_S390_R6;
        const R7 = 1 << bindings::PERF_REG_S390_R7;
        const R8 = 1 << bindings::PERF_REG_S390_R8;
        const R9 = 1 << bindings::PERF_REG_S390_R9;
        const R10 = 1 << bindings::PERF_REG_S390_R10;
        const R11 = 1 << bindings::PERF_REG_S390_R11;
        const R12 = 1 << bindings::PERF_REG_S390_R12;
        const R13 = 1 << bindings::PERF_REG_S390_R13;
        const R14 = 1 << bindings::PERF_REG_S390_R14;
        const R15 = 1 << bindings::PERF_REG_S390_R15;
        const FP0 = 1 << bindings::PERF_REG_S390_FP0;
        const FP1 = 1 << bindings::PERF_REG_S390_FP1;
        const FP2 = 1 << bindings::PERF_REG_S390_FP2;
        const FP3 = 1 << bindings::PERF_REG_S390_FP3;
        const FP4 = 1 << bindings::PERF_REG_S390_FP4;
        const FP5 = 1 << bindings::PERF_REG_S390_FP5;
        const FP6 = 1 << bindings::PERF_REG_S390_FP6;
        const FP7 = 1 << bindings::PERF_REG_S390_FP7;
        const FP8 = 1 << bindings::PERF_REG_S390_FP8;
        const FP9 = 1 << bindings::PERF_REG_S390_FP9;
        const FP10 = 1 << bindings::PERF_REG_S390_FP10;
        const FP11 = 1 << bindings::PERF_REG_S390_FP11;
        const FP12 = 1 << bindings::PERF_REG_S390_FP12;
        const FP13 = 1 << bindings::PERF_REG_S390_FP13;
        const FP14 = 1 << bindings::PERF_REG_S390_FP14;
        const FP15 = 1 << bindings::PERF_REG_S390_FP15;
        const MASK = 1 << bindings::PERF_REG_S390_MASK;
        const PC = 1 << bindings::PERF_REG_S390_PC;

    }
}

impl RegMask for S390RegMask {
    fn as_bits(&self) -> u64 {
        self.bits()
    }
}

bitflags! {
    /// Register masks for X86.
    pub struct X86RegMask : u64 {
        const AX = 1 << bindings::PERF_REG_X86_AX;
        const BX = 1 << bindings::PERF_REG_X86_BX;
        const CX = 1 << bindings::PERF_REG_X86_CX;
        const DX = 1 << bindings::PERF_REG_X86_DX;
        const SI = 1 << bindings::PERF_REG_X86_SI;
        const DI = 1 << bindings::PERF_REG_X86_DI;
        const BP = 1 << bindings::PERF_REG_X86_BP;
        const SP = 1 << bindings::PERF_REG_X86_SP;
        const IP = 1 << bindings::PERF_REG_X86_IP;
        const FLAGS = 1 << bindings::PERF_REG_X86_FLAGS;
        const CS = 1 << bindings::PERF_REG_X86_CS;
        const SS = 1 << bindings::PERF_REG_X86_SS;
        const DS = 1 << bindings::PERF_REG_X86_DS;
        const ES = 1 << bindings::PERF_REG_X86_ES;
        const FS = 1 << bindings::PERF_REG_X86_FS;
        const GS = 1 << bindings::PERF_REG_X86_GS;
        const R8 = 1 << bindings::PERF_REG_X86_R8;
        const R9 = 1 << bindings::PERF_REG_X86_R9;
        const R10 = 1 << bindings::PERF_REG_X86_R10;
        const R11 = 1 << bindings::PERF_REG_X86_R11;
        const R12 = 1 << bindings::PERF_REG_X86_R12;
        const R13 = 1 << bindings::PERF_REG_X86_R13;
        const R14 = 1 << bindings::PERF_REG_X86_R14;
        const R15 = 1 << bindings::PERF_REG_X86_R15;
        // XMM registers take up two slots since they are 128-bit registers.
        const XMM0 = 3 << bindings::PERF_REG_X86_XMM0;
        const XMM1 = 3 << bindings::PERF_REG_X86_XMM1;
        const XMM2 = 3 << bindings::PERF_REG_X86_XMM2;
        const XMM3 = 3 << bindings::PERF_REG_X86_XMM3;
        const XMM4 = 3 << bindings::PERF_REG_X86_XMM4;
        const XMM5 = 3 << bindings::PERF_REG_X86_XMM5;
        const XMM6 = 3 << bindings::PERF_REG_X86_XMM6;
        const XMM7 = 3 << bindings::PERF_REG_X86_XMM7;
        const XMM8 = 3 << bindings::PERF_REG_X86_XMM8;
        const XMM9 = 3 << bindings::PERF_REG_X86_XMM9;
        const XMM10 = 3 << bindings::PERF_REG_X86_XMM10;
        const XMM11 = 3 << bindings::PERF_REG_X86_XMM11;
        const XMM12 = 3 << bindings::PERF_REG_X86_XMM12;
        const XMM13 = 3 << bindings::PERF_REG_X86_XMM13;
        const XMM14 = 3 << bindings::PERF_REG_X86_XMM14;
        const XMM15 = 3 << bindings::PERF_REG_X86_XMM15;
    }
}

impl X86RegMask {
    /// Alias of [`AX`](Self::AX)
    pub const EAX: Self = Self::AX;
    /// Alias of [`BX`](Self::BX)
    pub const EBX: Self = Self::BX;
    /// Alias of [`CX`](Self::CX)
    pub const ECX: Self = Self::CX;
    /// Alias of [`DX`](Self::DX)
    pub const EDX: Self = Self::DX;
    /// Alias of [`SI`](Self::SI)
    pub const ESI: Self = Self::SI;
    /// Alias of [`DI`](Self::DI)
    pub const EDI: Self = Self::DI;
    /// Alias of [`BP`](Self::BP)
    pub const EBP: Self = Self::BP;
    /// Alias of [`SP`](Self::SP)
    pub const ESP: Self = Self::SP;
    /// Alias of [`IP`](Self::IP)
    pub const EIP: Self = Self::IP;

    /// Alias of [`AX`](Self::AX)
    pub const RAX: Self = Self::AX;
    /// Alias of [`BX`](Self::BX)
    pub const RBX: Self = Self::BX;
    /// Alias of [`CX`](Self::CX)
    pub const RCX: Self = Self::CX;
    /// Alias of [`DX`](Self::DX)
    pub const RDX: Self = Self::DX;
    /// Alias of [`SI`](Self::SI)
    pub const RSI: Self = Self::SI;
    /// Alias of [`DI`](Self::DI)
    pub const RDI: Self = Self::DI;
    /// Alias of [`BP`](Self::BP)
    pub const RBP: Self = Self::BP;
    /// Alias of [`SP`](Self::SP)
    pub const RSP: Self = Self::SP;
    /// Alias of [`IP`](Self::IP)
    pub const RIP: Self = Self::IP;
}

impl RegMask for X86RegMask {
    fn as_bits(&self) -> u64 {
        self.bits()
    }
}
