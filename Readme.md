# KFS
## Introduction

Kernel From Scratch, more likely know as KFS is a project from the cursus of [42](https://42.fr/), it is a project divided in gradual steps pushing towards a more and more sophisticated kernel.<br> The target arch is i386



## Current progress

### GDT + IDT
Functional GDT and IDT implementations featuring implemented interrupt handlers such as timer or keyboard. The groundwork has been done to support more handlers as needed

### Paging
Higher up mapped kernel, basic memory allocator suited for kernel level needs,  core::alloc::GlobalAlloc can be used

### Fun stuff
-	Basic shell, type help to get started
-	"Paint", CTRL + 2 to access a mode where move the cursor with arrow keys and type chars anywhere you want<br>CTRL + 1 to go back to the shell, buffer is saved upon exit
-	Shell command colors, allowing you to change the colors of foreground and background, this applies to the paint writes, so get painting
-	snake mode
-	Lots of non notable things, cursor, stack dump, shutdown, idle state, virtual memory toys in the shell, azerty layout and stuff that will come later on