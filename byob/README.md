# bring your own bootloader!
wukkOS is currently reliant on limine, and it is also currently required for you to
provide your own binaries for it. those binaries should be placed in this directory,
and the Makefile will copy them into the iso.  
currently the binaries are:
- limine.sys
- limine-cd.bin
- limine-cd-efi.bin
- limine-deploy (should be for your current system)