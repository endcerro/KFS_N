# KFS
## Introduction

Kernel From Scratch, more likely know as KFS is a project from the cursus of [42](https://42.fr/), it is a project divided in gradual steps pushing towards a more and more sophisticated kernel.<br> The target arch is i386



## Current progress

### GDT + IDT
Functional GDT and IDT implementations featuring implemented interrupt handlers such as timer or keyboard. The groundwork has been done to support more handlers as needed

### Paging
Higher up mapped kernel, basic memory allocator suited for kernel level needs,  core::alloc::GlobalAlloc can be used

### Fun stuff
-	Serial output
-	Basic shell, type help to get started
-	"Paint", CTRL + 2 to access a mode where move the cursor with arrow keys and type chars anywhere you want<br>CTRL + 1 to go back to the shell, buffer is saved upon exit
-	Shell command colors, allowing you to change the colors of foreground and background, this applies to the paint writes, so get painting
-	snake mode
-	Lots of non notable things, cursor, stack dump, shutdown, idle state, virtual memory toys in the shell, azerty layout and stuff that will come later on

### Running it

I've added a dockerfile for ease of use, provided you have make and qemu installed just run

```shell
make docker ; make qemu
```

If you don't you can read the docker rule in the makefile, you'll still need a virtual machine to run it (kvm qemu vbox etc)



I'm kind of underselling it but there is quite of bit of work i've done and I consider this a solid fundation for the work i'll do on this in the coming years