use crate::types::{VirtAddr, SVSM_CS};
use core::arch::{asm, global_asm};

#[repr(C, packed)]
pub struct x86_regs {
	r15		: u64,
	r14		: u64,
	r13		: u64,
	r12		: u64,
	r11		: u64,
	r10		: u64,
	r9		: u64,
	r8		: u64,
	rbp		: u64,
	rdi		: u64,
	rsi		: u64,
	rdx		: u64,
	rcx		: u64,
	rbx		: u64,
	rax		: u64,
	vector		: u64,
	error_code	: u64,
	rip		: u64,
	cs		: u64,
	flags		: u64,
	rsp		: u64,
	ss		: u64,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct IdtEntry {
	low	: u64,
	high	: u64,
}

const IDT_TARGET_MASK_1 : u64 = 0x0000_0000_0000_ffff;
const IDT_TARGET_MASK_2 : u64 = 0x0000_0000_ffff_0000;
const IDT_TARGET_MASK_3 : u64 = 0xffff_ffff_0000_0000;

const IDT_TARGET_MASK_1_SHIFT	: u64 = 0;
const IDT_TARGET_MASK_2_SHIFT	: u64 = 48 - 16;
const IDT_TARGET_MASK_3_SHIFT	: u64 = 32;

const IDT_TYPE_MASK		: u64 = 0xeu64 << 40; // Only interrupt gates for now
const IDT_PRESENT_MASK		: u64 = 1u64   << 47;
const IDT_CS_SHIFT 		: u64 = 16;

const IDT_IST_MASK		: u64 = 0x7;
const IDT_IST_SHIFT		: u64 = 32;

impl IdtEntry {
	const fn create(target : VirtAddr, cs : u16, ist : u8) -> Self {
		let vaddr = target as u64;
		let cs_mask = (cs as u64) << IDT_CS_SHIFT;
		let ist_mask = ((ist as u64) & IDT_IST_MASK) << IDT_IST_SHIFT;
		let low = (vaddr & IDT_TARGET_MASK_1) << IDT_TARGET_MASK_1_SHIFT |
			  (vaddr & IDT_TARGET_MASK_2) << IDT_TARGET_MASK_2_SHIFT |
			  IDT_TYPE_MASK | IDT_PRESENT_MASK | cs_mask | ist_mask;
		let high = (vaddr & IDT_TARGET_MASK_3) >> IDT_TARGET_MASK_3_SHIFT;

		IdtEntry { low : low, high : high }
	}

	pub const fn entry(target : VirtAddr) -> Self {
		IdtEntry::create(target, SVSM_CS, 0)
	}

	pub const fn no_handler() -> Self {
		IdtEntry { low : 0, high : 0 }
	}
}

const IDT_ENTRIES : usize = 256;

#[repr(C, packed)]
struct IdtDesc {
	size	: u16,
	address : usize,
}

extern "C" {
	static idt_handler_array : u8;
}

type Idt = [IdtEntry; IDT_ENTRIES];

static mut GLOBAL_IDT : Idt = [IdtEntry::no_handler(); IDT_ENTRIES];

fn init_idt(idt : &mut Idt) {
	// Set IDT handlers
	for i in 0..IDT_ENTRIES {
		unsafe {
			let handler = ((&idt_handler_array as *const u8) as VirtAddr) + (32 * i);
			idt[i] = IdtEntry::entry(handler);
		}
	}
}

fn load_idt(idt : &Idt) {
	let desc : IdtDesc = IdtDesc {
		size	: (IDT_ENTRIES * 16) as u16,
		address	: idt.as_ptr() as VirtAddr,
	};

	unsafe { asm!("lidt (%rax)", in("rax") &desc, options(att_syntax)); }
}

pub fn idt_init() {
	unsafe {
		init_idt(&mut GLOBAL_IDT);
		load_idt(&GLOBAL_IDT);
	}
}

#[no_mangle]
fn generic_idt_handler(regs : &mut x86_regs) {
	unsafe {
		asm!("12: jmp 12b", in("rax") regs.vector, in("rcx") regs.error_code, options(att_syntax));
	}
	loop { }
}

// Entry Code
global_asm!(r#"
		.text
	push_regs:
		pushq	%rax
		pushq	%rbx
		pushq	%rcx
		pushq	%rdx
		pushq	%rsi
		pushq	%rdi
		pushq	%rbp
		pushq	%r8
		pushq	%r9
		pushq	%r10
		pushq	%r11
		pushq	%r12
		pushq	%r13
		pushq	%r14
		pushq	%r15

		movq	%rsp, %rdi
		call	generic_idt_handler

		popq	%r15
		popq	%r14
		popq	%r13
		popq	%r12
		popq	%r11
		popq	%r10
		popq	%r9
		popq	%r8
		popq	%rbp
		popq	%rdi
		popq	%rsi
		popq	%rdx
		popq	%rcx
		popq	%rbx
		popq	%rax

		addq	$16, %rsp /* Skip vector and error code */

		iret

		.align 32
		.globl idt_handler_array
	idt_handler_array:
		i = 0
		.rept 32
		.align 32
		.if ((0x20027d00 >> i) & 1) == 0
		pushq	$0
		.endif
		pushq	$i	/* Vector Number */
		jmp	push_regs
		i = i + 1
		.endr
		"#,
		options(att_syntax));
