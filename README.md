This is the source code and patch repository for Linux host kernel, Qemu and OVMF to add Intel TDX (Trusted Domain eXtension) support to the COCONUT Secure VM Service Module (SVSM), a software which aims to provide secure services and device emulations to guest operating systems in confidential virtual machines (CVMs). The original COCONUT SVSM required AMD Secure Encrypted Virtualization with Secure Nested Paging (AMD SEV-SNP), especially the VM Privilege Level (VMPL) feature. This modification requires Intel TDX (Trusted Domain eXtension) technology, and it is early prototype code to collect feedback. It should only be used for architecture discussion, and NOT for any production related purpose.

The COCONUT-SVSM is distributed under the MIT license, which is included in the LICENSE-MIT file.
