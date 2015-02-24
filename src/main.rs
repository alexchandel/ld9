//! A crude, crude cross-linker from Mac to Plan 9. It reads a Mach-O into
//! memory, checks that it isn't dynamically linked, ASSUMES its text segment
//! is based at 0x1000 and its data at 0x2000, and blindly copies bytes into a
//! Plan 9 object.
//!
//! This is a proof-of-concept that executables can be cross-linked to Plan 9.
//! Programs should be compiled with Plan 9 syscalls, following the Plan 9 ABI
//! wherever possible. Mach-O is just the vehicle of compilation and linking,
//! permitting the use of LLVM and other nice tools.
//!
//! ## Limitations
//!
//! Where to begin. For one, ld9 assumes the entry point is the first
//! symbol in the TEXT. It assumes that the TEXT can/will be loaded to 0x1000,
//! despite the fact that by default static Mach-O's are compiled to load their
//! text to `DATA - size(TEXT)`; hope it's position-independent.
//!
//! It also assumes that the TEXT will find the DATA wherever Plan 9 loads it;
//! this actually is a workable assumption, since Mac loads it to 0x2000
//! by default, and Plan 9 loads it to "the first page-rounded virtual address
//! after the text segment", which is 0x2000.
//!
//! It goes without saying that this only supports and applies to static
//! executables, since by design Plan 9 does not support dynamic linking.

#![allow(non_snake_case, non_upper_case_globals)]
#![feature(core, io, fs)]

extern crate byteorder;

use std::mem::size_of;
use std::num::ToPrimitive;
use std::iter::AdditiveIterator;
use std::borrow::ToOwned;
use std::io::Read;

mod macho;
use macho::MachO;
mod aout9;
use aout9::AOut9;

#[derive(Debug)]
pub enum Error {
	TooShort,
	UnknownMagic(u32),
	SizeMismatch,
	UnrecognizedSegment(usize, u32),
	UnrecognizedThreadState(u32, u32),
	DynamicUnsupported,
}
use Error::*;

/// Treat a struct as an array of bytes.
// unsafe fn as_bytes<'a, T>(data: &'a mut T) -> &'a mut [u8] {
// 	std::slice::from_raw_parts_mut(data as *mut T as *mut u8,
// 		size_of::<T>())
// }

/// Reinterprets a slice as T.
/// Undefined behavior if slice is shorter than size of T.
unsafe fn reinterpret_copy<T>(data: &[u8]) -> T {
	let (rs, _) = std::mem::transmute::<&[u8],(&T, usize)>(data);
	std::mem::transmute_copy::<T, T>(rs)
}

/// Load a Mach-O file into memory.
fn load<T: ToPrimitive, U: ToPrimitive>(file: &[u8], offset: T, size: U)
-> Vec<u8> {
	let off = offset.to_uint().unwrap();
	let siz = size.to_uint().unwrap();
	file[off..off+siz].to_owned()
}

fn decode_macho(file: &[u8]) -> Result<MachO, Error> {
	use macho::*;

	let mut offset = 0;
	if file.len() < 4 {return Err(TooShort)}

	let h: Header = unsafe {reinterpret_copy(&file[..])};

	offset += match h.magic {
		M32 => size_of::<Header>(),
		M64 => size_of::<Header64>(),
		_ => return Err(UnknownMagic(h.magic.0))
	};

	if file.len() < offset + h.sizeofcmds as usize {return Err(TooShort)};

	let mut l = Vec::with_capacity(h.ncmds as usize);
	for i in range(0, h.ncmds as usize) {
		let lch: LoadCommandHead = unsafe {reinterpret_copy(&file[offset..])};
		let cmd_size = lch.cmdsize as usize;
		let seg: Result<LC, Error> = match lch.cmd {
			Segment32 => {
				println!("seg {}", cmd_size);
				let lc_size = size_of::<LoadCommand<LcSegment32>>();
				let lc: LoadCommand<LcSegment32> = unsafe {reinterpret_copy(
					&file[offset..])};
				let nsects = lc.body.nsects as usize;
				let section_size = size_of::<Section32>();
				let est_size = lc_size + section_size*nsects;
				if est_size != cmd_size {
					println!("{} vs {}", est_size, cmd_size);
					Err(SizeMismatch)
				} else {
					let mut sections = Vec::with_capacity(nsects);
					for j in range(0, nsects) {
						let sect: Section32 = unsafe {reinterpret_copy(
							&file[offset + lc_size + section_size*j ..])};
						let data = load(file, sect.offset, sect.size);
						sections.push((sect, data));
					}
					Ok(LC::Segment32(lc, sections))
				}
			},
			Segment64 => {
				println!("seg {}", cmd_size);
				let lc_size = size_of::<LoadCommand<LcSegment64>>();
				let lc: LoadCommand<LcSegment64> = unsafe {reinterpret_copy(
					&file[offset..])};
				let nsects = lc.body.nsects as usize;
				let section_size = size_of::<Section64>();
				let est_size = lc_size + section_size*nsects;
				if est_size != cmd_size {
					println!("{} vs {}", est_size, cmd_size);
					Err(SizeMismatch)
				} else {
					let mut sections = Vec::with_capacity(nsects);
					for j in range(0, nsects) {
						let sect: Section64 = unsafe {reinterpret_copy(
							&file[offset + lc_size + section_size*j ..])};
						let data = load(file, sect.offset, sect.size);
						sections.push((sect, data));
					}
					Ok(LC::Segment64(lc, sections))
				}
			},
			Symtab => {
				let lc: LoadCommand<LcSymtab> = unsafe {reinterpret_copy(
					&file[offset..])};
				Ok(LC::Symtab(lc))
			},
			DySymtab => {
				let lc: LoadCommand<LcDySymtab> = unsafe {reinterpret_copy(
					&file[offset..])};
				Ok(LC::DySymtab(lc))
			},
			Uuid => {
				let lc: LoadCommand<LcUuid> = unsafe {reinterpret_copy(
					&file[offset..])};
				Ok(LC::Uuid(lc))
			},
			VersionMinOS => {
				let lc: LoadCommand<LcVersionMinOS> = unsafe {reinterpret_copy(
					&file[offset..])};
				Ok(LC::VersionMinOS(lc))
			},
			SourceVersion => {
				let lc: LoadCommand<LcSourceVersion> = unsafe {reinterpret_copy(
					&file[offset..])};
				Ok(LC::SourceVersion(lc))
			},
			UnixThread => {
				let lc: LoadCommand<LcUnixThreadHead>
					= unsafe {reinterpret_copy(&file[offset..])};
				let lc_size = size_of::<LoadCommand<LcUnixThreadHead>>();
				match (lc.body.flavor, lc.body.count) {
					(ThreadStateFlavorX86, 16) => {
						let ts = ThreadState::ThreadStateX86(
							unsafe {reinterpret_copy(&file[offset+lc_size..])});
						Ok( LC::UnixThread(lc, ts))
					},
					(ThreadStateFlavorX86_64, 42) => {
						let ts = ThreadState::ThreadStateX86_64(
							unsafe {reinterpret_copy(&file[offset+lc_size..])});
						Ok(LC::UnixThread(lc, ts))
					},
					(f, c) => Err(UnrecognizedThreadState(f.0, c)),
				}
			},
			cmd => {Err(UnrecognizedSegment(i, cmd.0))},
		};
		match seg {
			Ok(s) => l.push(s),
			Err(e) => return Err(e),
		}
		offset += cmd_size;
	}

	Ok(MachO {header: h, loads: l})
}

// Write a Mach-O into A.out. 32-bit only
fn to_aout(m: &MachO) -> Result<AOut9, Error> {
	use macho::LC;
	use aout9::*;

	if m.is_dynamic() {return Err(DynamicUnsupported)};

	let text = m.loads.iter()
		.filter_map(|lc| match lc{
			&LC::Segment32(ref c, ref sects) => Some((c, sects)), _ => None})
		.filter(|&(c,_)| &c.body.segname[0..6] == b"__TEXT")
		.flat_map(|(_,sects)| sects.iter().flat_map(|&(ref __, ref d)| d.iter()))
		.cloned()
		.collect();

	let data = m.loads.iter()
		.filter_map(|lc| match lc{
			&LC::Segment32(ref c, ref sects) => Some((c, sects)), _ => None})
		.filter(|&(c,_)| &c.body.segname[0..6] == b"__DATA")
		.flat_map(|(_,sects)| sects.iter().flat_map(|&(ref __, ref d)| d.iter()))
		.cloned()
		.collect();

	let bss = m.loads.iter()
		.filter_map(|lc| match lc{
			&LC::Segment32(ref c, ref sects) => Some((c, sects)), _ => None})
		.filter(|&(c,_)| &c.body.segname[0..6] == b"__DATA")
		.flat_map(|(_,sects)| sects.iter())
		.filter(|sect| &sect.0.sectname[0..5] == b"__bss")
		.map(|sect| sect.0.size)
		.sum();

	Ok(AOut9 {
		magic: Magic::I,
		text: text,
		data: data,
		bss: bss as u64,
		entry: 0x20,
	})
}

fn main() {
	let mut f = std::fs::File::open("main").unwrap();
	let mut v = Vec::with_capacity(f.metadata().unwrap().len() as usize);
	f.read_to_end(&mut v).unwrap();

	let decoded = decode_macho(&v[..]).unwrap();
	let f = std::fs::File::create("aout9").unwrap();
	to_aout(&decoded).unwrap().write_to(f).unwrap();

	println!("{:x}", decoded.loads.len());
}
