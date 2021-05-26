# Rust Multiboot Binary (x86_64)

This project builds a minimal `multiboot2`-compatible binary (packed in an ELF-file [[2]]) that can be 
loaded by `GRUB` [[0]], a `multiboot2`-compliant [[1]] bootloader, that is written in Rust. The demo project focuses on the **x86_64** 
processor architecture and `UEFI` as firmware environment. After `multiboot2` handoff the processor is in 
`32-bit protected mode` and needs further steps to be able to execute 64-bit code and use 64-bit address space
(yes, x86 opcode is slightly different between `x86` and `x86_64` - can lead to weird behavior!). 
For simplicity reasons, it doesn't cover other architectures or legacy BIOS. 

This means, the final binary is a mix of `32-bit` `x86_64` (aka `x86` aka `i686`) code and `64-bit` `x86_64` code.
The final binary gets assembled from the three object files:
- `multiboot2_header.asm`
- `start.asm`
- the Rust binary (TODO so far only 32-bit, no handoff and 64-bit code yet)

**⚠️ ATTENTION: current state: no transition to 64-bit mode yet, only verifying that the binary got booted via `multiboot2` ⚠️**

Inside this repository you will find `build.sh` and `run_qemu.sh`, which enables you to "see" what's happening and to test everything.
**There are plenty of comments in the shell scripts and the code, therefore this `README` only gives a high-level overview.**

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
(`multiboot2` handoff). When `GRUB` is done loading our `64-bit` (or `32-bit`, both works) `ELF`-file, and our
application starts executing, a CPU core can subsequently execute the code. At this point we only have one single 
core available that is in `32-bit protected mode` (even on multicore `x86_64` processors). We need further 
setup in future steps to use `64-bit` mode (*longmode*) and other cores. 

`multiboot2` stores the pointer to a data structure that gets passed to us via `ebx` register and stores a magic number
in `eax`. This must be done at the very beginning of the Rust binary, because otherwise `eax` or `ebx` may get 
overwritten. In data structure mentioned in the first sentence of the paragraph, we can for example find the memory 
map of `UEFI` and interact with `UEFI`- functions.

#### UEFI vs BIOS
For `multiboot2`-compliant binaries it's irrelevant whether the firmware
is a legacy `BIOS` or an `UEFI`-implementation. What's relevant is that the binary 
gets a `multiboot2` structure passed as payload, that contains multiple "tags".
A tag can for example contain the "memory map" of the `UEFI` [[1]].

Therefore, a "100% bullet proof" bootloader binary must cope with two kinds of firmware (legacy BIOS and UEFI) 
(e.g. update the firmware via the OS). In this example, for the sake of simplicity and because legacy sucks, 
I only focus on `UEFI`.

When our binary starts, the `UEFI boot time services` are still available. Consult the `UEFI`-spec for 
further information. This means, that it is up to our binary to exit the boot services.

#### UEFI: how to use runtime and boot services
TODO: switch to 64-bit long mode first? any why? better explanations here!

#### Processor Mode
Luckily, `GRUB`  makes some CPU initialization, which we don't have to do here. This is required
by the `multiboot(2)`-spec, that is implemented by `GRUB`. For example, it sets the CPU from `16-bit 
real mode` into `32-bit protected mode` [[1]] (remember, we still only have a single core, that is set up).
Even tho `GRUB` can load `64-bit`-`ELF`-files, the processor
will not be in `64-bit` mode after handoff [[1]]. 

⚠️Attention here: When you have a `64-bit` `ELF`-file, the entry function still must be `32-bit` code!
Fortunately, both can coexist inside the same `ELF`. Executing `64-bit` code in `32-bit` mode results
in strange behaviour/wrong instructions/opcodes (I ran into this problem!).

Therefore, it is important to compile the binary (at least the very startup code after `multiboot2`-handoff) 
as `32-bit` `x86` code. For a "good software design" you could write another binary, that compiles to `64-bit` code 
and handoff to it after you initialized`64-bit` mode. Or you link `32-bit` and `64-bit` code into the same `ELF` 
and handover to code by hand when the time is right.

Consult the spec [[1]] for more detailed information about the system state after handoff.

TODO: switch to `64-bit` ("long") mode.

#### Stack
We need to set up our own stack, `multiboot2` or `UEFI` don't cover this. 
For simplicity, I have written a `start.asm`, which defines the very first entrypoint
called `start`. This entry point is referenced in `linked.ld`. After the stack is 
initialized (note, that we have no stack protection yet!), it directly handsoff to
`_start` symbol defined by the Rust binary.

#### Memory
- TODO: how many memory does our binary use? how much can it use at maximum (to not overwrite data from grub for example?)
- 

---

### Running & Debugging in Qemu
#### Running
`QEMU` itself has a `-kernel`-option to boot a `multiboot`-kernel. Unfortunately,
this doesn't support `multiboot2` but we have a `multiboot2` binary/kernel here.
Therefore, I took the approach of loading `edk2/OVMF` (`UEFI`) into `QEMU`, which itself loads the `GRUB` binary.
`GRUB` will finally load the Rust binary. This way, I can load/boot the `multiboot2`-compatible ELF binary 
that is written in Rust.

#### Getting Output (in QEMU)
It's hard to get output because at this point we can't easily draw to GUI or print to STDOUT.. :)

##### Getting Output A: Change register values and `halt`
You can write a value into a register and halt. In the QEMU GUI, you can open 
`View > compatmonitor0` and type `info registers`- you should see the value!

```rust
unsafe { asm!("mov edi, {val}", "hlt", val = const 0x0bad_f00d_u32); x86::halt() };
```

##### Getting Output B: Use Qemu
- TODO, this section needs TODO (didn't worked for me yet)
- Qemu has a debug port called "debug conn"
- with it, the application can use `out al, 0xe9` (x86 IO port)
- Qemu can map this debug connection to a file or stdout

##### Getting Output C: GDB
- TODO

---

## Trivia/FAQ/Good to know
- `multiboot(2)` only specifies behavior for x86 but not for other architectures, like ARM

## Open Questions / TODO
- [ ] Heap inside application?!
- [ ] move linker script into `x86_64-none-multiboot2_elf.json` (link args etc?!)
- [x] do I have to change the final ELF to 32 bit?! because multiboot2 doesn't say much about 64 bit support
  - probably no, because grub can load it anyway (it still needs to switch to long mode (64bit))
- [ ] why does debug conn doesn't work?!
- [ ] How to ensure in Linker Script, that no code is mapped to address
      where UEFI stuff is stored?
- [ ] remove `nasm` assembly and use assembly language that "rustc" can compile
- [ ] am I in i386 mode or amd64 mode with elf 64?! / move again to 64-bit elf but with 32-bit startup code
- [ ] build on other platforms; Mac or Windows?! Does it link an ELF file there too?


## References
[0]: https://www.gnu.org/software/grub/
[1]: https://www.gnu.org/software/grub/manual/multiboot2/
[2]: https://refspecs.linuxfoundation.org/elf/elf.pdf
[3]: https://github.com/tianocore/edk2
[4]: https://uefi.org/sites/default/files/resources/UEFI_Spec_2_9_2021_03_18.pdf


