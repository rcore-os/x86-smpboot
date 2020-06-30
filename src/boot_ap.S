# Boot application processors.

# Each non-boot CPU ("AP") is started up in response to a STARTUP
# IPI from the boot CPU.  Section B.4.2 of the Multi-Processor
# Specification says that the AP will start in real mode with CS:IP
# set to XY00:0000, where XY is an 8-bit value sent with the
# STARTUP. Thus this code must start at a 4096-byte boundary.
#
# Because this code sets DS to zero, it must sit
# at an address in the low 2^16 bytes.

.equ ap_start64_paddr, ap_start64 - ap_start + 0x6000
.equ gdt_64_paddr, gdt_64 - ap_start + 0x6000
.equ gdt_64_pointer_paddr, gdt_64_pointer - ap_start + 0x6000

.equ cr3_ptr, 0x6ff8
.equ entry_ptr, 0x6ff0
.equ stack_ptr, 0x6fe8
.equ temp_stack_top, 0x6fe0

.global ap_start
.global ap_end

.text
.intel_syntax noprefix
.code16
ap_start:
    cli

    xor     ax, ax
    mov     ds, ax
    mov     es, ax
    mov     ss, ax

    # set PAE, PGE, OSFXSR, OSXMMEXCPT bit
    mov     eax, cr4
    or      eax, (1 << 5) | (1 << 7) | (1 << 9) | (1 << 10)
    mov     cr4, eax

    # load cr3
    mov     eax, [cr3_ptr]
    mov     cr3, eax

    # set LME, NXE bit
    mov     ecx, 0xC0000080
    rdmsr
    or      eax, (1 << 8) | (1 << 11)
    wrmsr

    # set protected mode, monitor co-processor, paging bit
    mov     eax, cr0
    or      eax, (1 << 0) | (1 << 1) | (1 << 31)
    mov     cr0, eax

    # set temporary stack
    mov     esp, temp_stack_top

    # load the 64-bit GDT
    lgdt    [gdt_64_pointer_paddr]

    # jump to long mode
    push    0x8
    lea     eax, [ap_start64_paddr]
    push    eax
    retf

.code64
ap_start64:
    # load 0 into all data segment registers
    xor     ax, ax
    mov     ss, ax
    mov     ds, ax
    mov     es, ax
    mov     fs, ax
    mov     gs, ax

    # set stack and jump to Rust code
    mov     rsp, [stack_ptr]
    mov     rax, [entry_ptr]
    call    rax

spin_hlt:
    hlt
    jmp     spin_hlt

gdt_64:
    .quad 0x0000000000000000            # Null Descriptor - should be present.
    .quad 0x00209A0000000000            # 64-bit code descriptor (exec/read).
    .quad 0x0000920000000000            # 64-bit data descriptor (read/write).
.align 4
    .word 0                             # Padding to make the "address of the GDT" field aligned on a 4-byte boundary

gdt_64_pointer:
    .word gdt_64_pointer - gdt_64 - 1   # 16-bit Size (Limit) of GDT.
    .long gdt_64_paddr                  # 32-bit Base Address of GDT. (CPU will zero extend to 64-bit)

ap_end:
