# Rust Multiboot2 Binary (Kernel) (x86_64)

This project builds a minimal `multiboot2`-compatible [[1]] binary (kernel) where the main logic is written in Rust. 
**"multiboot"** refers to a specification [[1]] that defines the handoff from a bootloader to a payload. It has nothing
to do with "multiple OS boot environments"! The binary gets packaged as an `ELF64-x86_64`-file [[2]], that can be loaded 
by `GRUB` [[0]], a`multiboot2`-compliant bootloader. The demo project focuses on the **x86_64** processor architecture 
and `UEFI` as firmware environment. After `multiboot2` handoff, the processor is in `64-bit long mode`.

To be exact, the entry point into the program is written in assembly, but the logic is minimal and the handoff
to the code produced by the Rust compiler follows shortly. It's the responsibility of the Rust code, to cope with the 
`multiboot2` payload (multiboot information structure) and all the system setup. The final binary gets assembled from
multiple object files: 

- `multiboot2_header.asm -> multiboot2_header.o` 
- `start.asm => start.o`
- the Rust binary (output of Cargo)

The **GNU linker** `ld` uses the linker script in `src/linker.ld` and the object files to assemble the final ELF binary. 
Inside this repository you will find `build.sh` and `run_qemu.sh`, which enables you to "see" what's happening and to 
test everything. **There are plenty of comments in the shell scripts and the code, therefore this `README` only gives a 
high-level overview.**

When you test this project (`run_qemu.sh`), it will
1) start QEMU + loads `edk2/OVMF` [[3]] as `UEFI`-environment [[4]] 
2) `OVMF` will automatically boot `GRUB` (an EFI file)
3) the `GRUB`-EFI-file has a `grub.cfg` file and the Rust binary built-in into the `GRUB`-internal `(memdisk)`-filesystem.
4) `GRUB` loads the cfg-file which starts the binary


With a boot-order of `firmware > GRUB > <my-binary>`, the binary could take the role of:
* another bootloader (multistage boot)
* an OS-Kernel written in Rust 
* an OS-specific loader which prepares hand-off to an OS-kernel (to decouple large software into smaller blocks of responsibility)

---

### Software/Tool versions 
- I tested the built on an Ubuntu 20.04 system with Linux 5.8.0
  - probably doesn't build on other platforms; it requires linkers that produce ELF files
- Rust-binary built using **Rust** `1.54-nightly`.
- `*.asm`-compilation compilation tested with **nasm** `2.14.02`
- linking the object files together was tested with **GNU ld** `2.34`
- "run in QEMU"-demonstration tested with
  - **GRUB**: 2.04-1ubuntu26.1
  - **QEMU**: 4.2.1

---

### System Environment & Processor Mode after handoff
This section should clarify how the Rust application sees the environment after the Rust binary was loaded
(`multiboot2` handoff). When `GRUB` is done loading our `ELF64-x86_64`-file, and our
application starts executing, a CPU core can subsequently execute the code. At this point we only have one single 
core (*"boot processor"*) available that is in `64-bit long mode` (even on multicore `x86_64` chips). We need
further setup to use other cores. If we boot other cores, they are first in `16-bit real mode`, then we must switch
them to `32-bit protected mode` after they finally will end in `64-bit long mode`.

Because we instruct the `multiboot2`-compliant bootloader `GRUB` to gives us the `AMD64`-machine state after handoff
(which is a synonym for `x86_64`), we don't have to cope with this on the boot processor because `GRUB` set it up 
for us - that is nice and convenient! Right before handoff to our binary, `GRUB` stores the pointer to the
multiboot information structure for us in register `ebx` and a magic number in `eax`. 

#### UEFI vs BIOS
For `multiboot2`-compliant binaries it's irrelevant whether the firmware
is a legacy `BIOS` or an `UEFI`-implementation. What's relevant is that the binary 
gets a `multiboot2` structure passed as payload, that contains multiple "tags".
A tag can for example contain the "memory map" of the `UEFI` [[1]].

Therefore, a "100% bullet proof" bootloader binary must cope with two kinds of firmware (legacy BIOS and UEFI). In this
example, for the sake of simplicity and because legacy sucks, I only focus on `UEFI`.  When our binary starts, the 
`UEFI boot time services` are still available. Consult the `UEFI`-spec for further information. This means, that it is 
up to our binary to exit the boot services.

#### Processor Mode
Luckily, `GRUB`  makes some CPU initialization, which we don't have to do here. This is required
by the `multiboot(2)`-spec, that is implemented by `GRUB`. For example, it sets the CPU from `16-bit 
real mode` into `32-bit protected mode` [[1]] (remember, we still only have a single core, that is set up).
Further, `GRUB` will also switch the boot processor (core) to `64-bit long mode`, as I said earlier already.
This works, because I specified the right ***tags*** in `src/multiboot2_header.asm`. Otherwise, we would be in 
`32-bit protected mode`. To be exactly, our system state is defined by section 3.5 of multiboot2 spec [[1]].
The multiboot2 spec refers to section 2.3.4 of UEFI spec 2.6. Incomplete list:

- 64-bit long mode
- paging enabled, all memory related to UEFI is identity mapped
- interrupts are enabled


#### Stack
I'm not 100% sure here. UEFI gives us a 128KiB stack according to the spec [[4]], but I think `GRUB` uses it already.
Therefore I define my own stack in `start.asm`.

#### Memory
- TODO: how many memory does our binary use? how much can it use at maximum (to not overwrite data from grub for example?)
- TODO how to allocate?

---

### Running & Debugging in Qemu
#### Running
`QEMU` itself has a `-kernel`-option to boot a `multiboot`-kernel. Unfortunately, this doesn't support `multiboot2` 
but we have a `multiboot2` binary/kernel here. Therefore, I took the approach of loading `edk2/OVMF` (`UEFI`) into 
`QEMU`, which itself loads the `GRUB`-binary. `GRUB` will finally load the `multiboot2` binary. The entry point will
be called, which is specified by the assembly code in `start.asm`.

#### Getting Output (in QEMU) & Debug
It's hard to get output because at this point we can't easily draw to GUI or print to STDOUT.. :)

##### Output & Debug A: Change register values and `halt`
You can write a value into a register and halt. In the QEMU GUI, you can open 
`View > compatmonitor0` and type `info registers`- you should see the register values!
Use this code for example:

```rust
unsafe { asm!("mov edi, {val}", "hlt", val = const 0x0bad_f00d_u32); x86::halt() };
```

##### Output & Debug B: Use QEMUs 'debugconn'
- TODO, this section needs TODO (didn't work for me yet)
- Qemu has a debug port called "debug conn"
- with it, the application can use `out al, 0xe9` (x86 IO port)
- Qemu can map this debug connection to a file or stdout

##### Output & Debug  C: GDB
- TODO

---

## Trivia/FAQ/Good to know/What I've learnt
- Q: Are OPCODES between 32-bit and 64-bit code different?
    - A: yes, I ran into this and learned it the hard way. If you execute 64-bit code in an 32-bit environment
         or vice versa, strange things will happen.
- `multiboot(2)` only specifies behavior for x86 but not for other architectures, like ARM
- Q: Why is the Rust binary a standalone library and not an executable?
    - A: The final binary gets assembled from multiple object files. Code must be relocatable by the linker,
         otherwise (relative) jumps and loads may get damaged.

## Open Questions / TODO
- [ ] Heap inside application?!
- [ ] move linker script into `x86_64-none-multiboot2_elf.json` (link args etc?!)
- [x] do I have to change the final ELF to 32 bit?! because multiboot2 doesn't say much about 64 bit support
  - probably no, because grub can load it anyway (it still needs to switch to long mode (64bit))
- [ ] How to ensure in Linker Script, that no code is mapped to address
      where UEFI stuff is stored?
- [ ] Debug and Prod build
- [ ] build a relocatable final binary to ensure that no code or data can overwrite uefi memory


## References
[0]: https://www.gnu.org/software/grub/
[1]: https://www.gnu.org/software/grub/manual/multiboot2/
[2]: https://refspecs.linuxfoundation.org/elf/elf.pdf
[3]: https://github.com/tianocore/edk2
[4]: https://uefi.org/sites/default/files/resources/UEFI_Spec_2_9_2021_03_18.pdf


