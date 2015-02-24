#![allow(non_upper_case_globals, non_camel_case_types, dead_code)]

use std::mem::size_of;
use std::slice::from_raw_parts;
use std::io::{Write, Result};

use byteorder::{BigEndian, WriteBytesExt};

macro_rules! magic(
	($f:expr, $b:expr) => (($f)|((((4*($b))+0)*($b))+7))
);

const Hdr: u32	= 0x00008000;

#[derive(Copy)]
#[repr(u32)]
pub enum Magic {
	A		= magic!(0, 8), /* 68020 */
	I		= magic!(0, 11), /* intel 386 */
	J		= magic!(0, 12), /* intel 960 (retired) */
	K		= magic!(0, 13), /* sparc */
	V		= magic!(0, 16), /* mips 3000 BE */
	X		= magic!(0, 17), /* att dsp 3210 (retired) */
	M		= magic!(0, 18), /* mips 4000 BE */
	D		= magic!(0, 19), /* amd 29000 (retired) */
	E		= magic!(0, 20), /* arm */
	Q		= magic!(0, 21), /* powerpc */
	N		= magic!(0, 22), /* mips 4000 LE */
	L		= magic!(0, 23), /* dec alpha */
	P		= magic!(0, 24), /* mips 3000 LE */
	U		= magic!(0, 25), /* sparc64 */
	S		= magic!(Hdr, 26), /* amd64 */
	T		= magic!(Hdr, 27), /* powerpc64 */
	R		= magic!(Hdr, 28), /* arm64 */
}


/// An executable Plan 9 binary file has up to six sections: a
/// header, the program text, the data, a symbol table, a PC/SP
/// offset table (MC68020 only), and finally a PC/line number
/// table.  The header, given by a structure in <a.out.h>, con-
/// tains 4-byte integers in big-endian order.
///
/// Entry is a virtual memory address, and must be precognizant of where the
/// header+text will be loaded. If the first symbol is the entry point, and the
/// header+text is loaded to 0x1000, then entry = 0x1020.
#[repr(packed)]
pub struct Header {
	magic:	u32,		/* magic number */
	text:	u32,		/* size of text segment */
	data:	u32,		/* size of initialized data */
	bss:	u32,		/* size of uninitialized data */
	syms:	u32,		/* size of symbol table */
	entry:	u32,		/* entry point */
	spsz:	u32,		/* size of pc/sp offset table */
	pcsz:	u32,		/* size of pc/line number table */
}

impl Header {
	fn to_be(&self) -> [u32; 8] {
		use std::intrinsics::bswap32;
		unsafe {[
			bswap32(self.magic),
			bswap32(self.text),
			bswap32(self.data),
			bswap32(self.bss),
			bswap32(self.syms),
			bswap32(self.entry),
			bswap32(self.spsz),
			bswap32(self.pcsz),
		]}
	}
}

/// Header, text, data, symbols, PC/SP, PC/SZ.
///
/// The symbol, PC/SP, and PC/SZ tables are not supported. That is, this is
/// a stripped object.
pub struct AOut9 {
	pub magic: Magic,
	pub text: Vec<u8>,
	pub data: Vec<u8>,
	pub bss: u64,
	pub entry: u64,
}

impl AOut9 {
	pub fn write_to<T: Write>(&self, mut sink: T) -> Result<()> {
		let header = Header {
			magic: self.magic as u32,
			text: self.text.len() as u32,
			data: self.data.len() as u32,
			bss: self.bss as u32,
			syms: 0,
			entry: self.entry as u32 + 0x1000,
			spsz: 0,
			pcsz: 0,
		};

		let h = unsafe {from_raw_parts(
			&header as *const Header as *const u32,
			size_of::<Header>() / size_of::<u32>()
		)};

		for dword in h {
			sink.write_u32::<BigEndian>(*dword).unwrap();
		}

		let result =      sink.write_all(&self.text[..])
			.and_then(|_| sink.write_all(&self.data[..]));

		result
	}
}

