//! All enums have an `Other` variant for pattern matching purposes. Never
//! compare against this.

#![allow(non_upper_case_globals, non_camel_case_types, dead_code)]

pub type VmProt = u32; // c_int
// pub type U8N16 = ((u8,u8,u8,u8, u8,u8,u8,u8), (u8,u8,u8,u8, u8,u8,u8,u8,));
pub type U8N16 = [u8; 16];

/// Indicates a 32 or 64-bit Mach-O file.
#[derive(PartialEq, Eq)]
#[repr(packed)]
pub struct Magic(pub u32);
pub const M32: Magic = Magic(0xfeedface);
pub const M64: Magic = Magic(0xfeedfacf);
impl Magic {
	#[inline(always)]
	fn is_valid(&self) -> bool {
		[M32, M64].contains(self)
	}
}


const ArchAbi64: u32	= 0x01_000000;
/// Indicates the architecture you intend to use the file on.
#[repr(u32)]
pub enum CpuType {
	X86			= 7,
	X86_64		= 7|ArchAbi64,
	PowerPc		= 18,
	PowerPc64	= 18|ArchAbi64,
}

/// The exact model of the CPU.
#[repr(u32)]
pub enum CpuSubtype {
	Any = -1i32 as u32,

	I386_All	= 3,

	// X86_All		= 3,
	// X86_64_All	= 3,
	X86_64h_All	= 8,
}

/// The purpose of the file.
#[repr(u32)]
pub enum Filetype {
	Object		= 0x1,
	Execute		= 0x2,
}

/// Certain optional features.
#[repr(u32)]
pub enum Flags {
	Noundefs	= 0x1,
	IncrLink	= 0x2,
	DyldLink	= 0x3,
}

/// ncmds := the number of load commands following the header
/// sizeofcmds := the number of bytes occupied by the load commands
#[repr(packed)]
pub struct Header {
	pub magic: Magic,
	pub cputype: CpuType, // c_int
	pub cpusubtype: CpuSubtype, // c_int
	pub filetype: Filetype,
	pub ncmds: u32,
	pub sizeofcmds: u32,
	pub flags: Flags,
	// pub reserved: (),
}

#[repr(packed)]
pub struct Header64 {
	pub magic: Magic,
	pub cputype: CpuType, // c_int
	pub cpusubtype: CpuSubtype, // c_int
	pub filetype: Filetype,
	pub ncmds: u32,
	pub sizeofcmds: u32,
	pub flags: Flags,
	pub reserved: u32,
}

#[derive(PartialEq)]
#[repr(packed)]
pub struct LoadCommandType(pub u32);
pub const Segment32:	LoadCommandType = LoadCommandType(0x1);
pub const Symtab:		LoadCommandType = LoadCommandType(0x2);
pub const UnixThread:	LoadCommandType = LoadCommandType(0x5);
pub const DySymtab:		LoadCommandType = LoadCommandType(0xb);
pub const LoadDylinker:	LoadCommandType = LoadCommandType(0xe);
pub const Segment64:	LoadCommandType = LoadCommandType(0x19);
pub const Uuid:			LoadCommandType = LoadCommandType(0x1b);
pub const VersionMinOS:	LoadCommandType = LoadCommandType(0x24);
pub const SourceVersion:	LoadCommandType = LoadCommandType(0x2A);

/// File offsets are from the absolute beginning of the file.
#[repr(packed)]
pub struct LcSegment32 {
	pub segname: U8N16,
	pub vmaddr: u32,
	pub vmsize: u32,
	pub fileoff: u32,
	pub filesize: u32,
	pub maxprot: VmProt,
	pub initprot: VmProt,
	pub nsects: u32,
	pub flags: u32,
}

/// File offsets are from the absolute beginning of the file.
#[repr(packed)]
pub struct LcSegment64 {
	pub segname: U8N16,
	pub vmaddr: u64,
	pub vmsize: u64,
	pub fileoff: u64,
	pub filesize: u64,
	pub maxprot: VmProt,
	pub initprot: VmProt,
	pub nsects: u32,
	pub flags: u32,
}

#[repr(packed)]
pub struct LcSymtab {
	symoff: u32,
	nsyms: u32,
	stroff: u32,
	strsize: u32,
}

#[derive(Copy)]
#[repr(packed)]
pub struct ThreadStateFlavor(pub u32);
pub const ThreadStateFlavorX86: ThreadStateFlavor = ThreadStateFlavor(1);
pub const ThreadStateFlavorX86_64: ThreadStateFlavor = ThreadStateFlavor(4);
pub enum ThreadState {
	ThreadStateX86([u32; 16]),
	ThreadStateX86_64([u64; 21]),
}

#[derive(Copy)]
#[repr(packed)]
pub struct LcUnixThreadHead {
	/// The architecture-flavor of thrad state data.
	pub flavor: ThreadStateFlavor,
	/// Size of thread state data in u32s. Data must be 32-bit aligned.
	pub count: u32,
}

#[repr(packed)]
pub struct LcDySymtab {
	/// Index of first symbol in the local symbols.
	ilocalsym: u32,
	/// Number of symbols in the local symbols.
	nlocalsym: u32,

	iextdefsym: u32,
	nextdefsym: u32,

	iundefsym: u32,
	nundefsym: u32,

	tocoff: u32,
	ntoc: u32,

	modtaboff: u32,
	nmodtab: u32,

	extrefsymoff: u32,
	nextrefsyms: u32,

	indirectsymoff: u32,
	nindirectsyms: u32,

	extreloff: u32,
	nextrel: u32,

	locreloff: u32,
	nlocrel: u32,
}

#[repr(packed)]
pub struct LcStr {
	offset: u32,
}

#[repr(packed)]
pub struct LcLoadDylinker {
	name: LcStr,
}

#[repr(packed)]
pub struct LcUuid {
	uuid: U8N16,
}

#[repr(packed)]
pub struct LcVersionMinOS {
	version: u32,
	sdk: u32,
}

#[repr(packed)]
pub struct LcSourceVersion {
	/// A.B.C.D.E packed as a24.b10.c10.d10.e10
	version: u64
}

use std::marker::MarkerTrait;
pub trait LoadCommandBody: MarkerTrait {}
impl LoadCommandBody for LcSegment32 {}
impl LoadCommandBody for LcSegment64 {}
impl LoadCommandBody for LcSymtab {}
impl LoadCommandBody for LcUnixThreadHead {}
impl LoadCommandBody for LcDySymtab {}
impl LoadCommandBody for LcLoadDylinker {}
impl LoadCommandBody for LcUuid {}
impl LoadCommandBody for LcVersionMinOS {}
impl LoadCommandBody for LcSourceVersion {}


#[repr(packed)]
pub struct LoadCommandHead {
	pub cmd: LoadCommandType,
	pub cmdsize: u32,
}

#[repr(packed)]
pub struct LoadCommand<T: LoadCommandBody> {
	pub head: LoadCommandHead,
	pub body: T,
}

#[repr(packed)]
pub struct Section32 {
	pub sectname: U8N16,
	pub segname: U8N16,
	pub addr: u32,
	pub size: u32,
	pub offset: u32,
	pub align: u32,
	pub reloff: u32,
	pub nreloc: u32,
	pub flags: u32,
	pub reserved1: u32,
	pub reserved2: u32,
}

#[repr(packed)]
pub struct Section64 {
	pub sectname: U8N16,
	pub segname: U8N16,
	pub addr: u64,
	pub size: u64,
	pub offset: u32,
	pub align: u32,
	pub reloff: u32,
	pub nreloc: u32,
	/// Section type and attributes
	pub flags: u32,
	/// (for offset or index)
	pub reserved1: u32,
	/// for count or sizeof)
	pub reserved2: u32,
	pub reserved3: u32,
}

/// A complete Mach-O segment, including any trailing sections and file data.
pub enum LC {
	Segment32(LoadCommand<LcSegment32>, Vec<(Section32, Vec<u8>)>),
	Symtab(LoadCommand<LcSymtab>),
	UnixThread(LoadCommand<LcUnixThreadHead>, ThreadState),
	DySymtab(LoadCommand<LcDySymtab>),
	LoadDylinker(LoadCommand<LcLoadDylinker>, Vec<u8>),
	Segment64(LoadCommand<LcSegment64>, Vec<(Section64, Vec<u8>)>),
	Uuid(LoadCommand<LcUuid>),
	VersionMinOS(LoadCommand<LcVersionMinOS>),
	SourceVersion(LoadCommand<LcSourceVersion>),
}

/// A Mach-O file loaded in memory.
pub struct MachO {
	pub header: Header,
	pub loads: Vec<LC>,
}

impl MachO {
	pub fn is_dynamic(&self) -> bool {
		self.loads.iter().any(|lc| match lc {
			&LC::DySymtab(_) => true,
			&LC::LoadDylinker(_, _) => true,
			_ => false,
		})
	}
}
