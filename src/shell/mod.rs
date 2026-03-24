use crate::keyboard;
use crate::keyboard::*;
use crate::vga;
use crate::vga::Color;
pub mod commands;
use crate::vga::get_current_colors;
use commands::{paint, *};

const MAX_COMMANDS: usize = 20;
const _MAX_COMMAND_LENGTH: usize = 20;
const MAX_ARGS: usize = 10;
static SHELL_ID: &str = "kernel@ring0:/#";
#[derive(Clone, Copy)]
struct Command {
    name: &'static str,
    function: fn(&[&str]),
    description: &'static str,
}

pub struct Shell {
    commands: [Command; MAX_COMMANDS],
    command_count: usize,
}

impl Shell {
    const fn new() -> Self {
        Shell {
            commands: [Command {
                name: "",
                function: Shell::dummy_command,
                description: "",
            }; MAX_COMMANDS],
            command_count: 0,
        }
    }

    fn add_command(
        &mut self,
        name: &'static str,
        function: fn(&[&str]),
        description: &'static str,
    ) {
        if self.command_count < MAX_COMMANDS {
            self.commands[self.command_count] = Command {
                name,
                function,
                description,
            };
            self.command_count += 1;
        }
    }

    pub fn run_command(&self, input: &str) {
        let mut parts = [""; MAX_ARGS];
        let mut part_count = 0;
        for (i, part) in input.split_whitespace().enumerate() {
            if i < MAX_ARGS {
                parts[i] = part;
                part_count += 1;
            } else {
                break;
            }
        }

        if part_count > 0 {
            let command = parts[0];
            let args = &parts[1..part_count];

            for cmd in &self.commands[0..self.command_count] {
                if cmd.name == command {
                    (cmd.function)(args);
                    return;
                }
            }
            hello_shell();
            print!("Unknown command: {}", command);
        }
    }

    fn dummy_command(_: &[&str]) {
        // This function should never be called
    }
}

pub fn init_shell() {
    unsafe {
        SHELL.add_command("echo", echo::run, "Echo the input arguments");
        SHELL.add_command("clear", clear::run, "Clear the screen");
        SHELL.add_command("credits", credits::run, "Credits");
        SHELL.add_command("colors", colors::run, "Change terminal colors");
        SHELL.add_command("ft", print_ft_42::run, "Print 42 logo");
        SHELL.add_command("stack", print_stack::run, "Print stack information");
        SHELL.add_command("help", help, "Display help information");
        SHELL.add_command(
            "meminfo",
            meminfo::run,
            "Displays memory mappings from multiboot2",
        );
        SHELL.add_command("exit", shutdown::run, "Shutdown");
        SHELL.add_command(
            "vmalloc",
            vmalloc::run,
            "Map virtual pages:    vmalloc <addr> <size>",
        );
        SHELL.add_command(
            "vfree",
            vfree::run,
            "Unmap virtual pages:  vfree <addr> <size>",
        );
        SHELL.add_command("vsize", vsize::run, "Query mapped size:    vsize <addr>");
        SHELL.add_command(
            "vwrite",
            vwrite::run,
            "Write to virt addr:   vwrite <addr> <val> [u8|u32|u64]",
        );
        SHELL.add_command(
            "vread",
            vread::run,
            "Read from virt addr:  vread <addr> [u8|u32|u64]",
        );
        SHELL.add_command("setkb", setkb::run, "Swap from QWERTY to AZERTY and back");
    }
}

pub fn hello_shell() {
    let mut color_pair: (Option<Color>, Option<Color>) = (None, None);
    if get_current_colors() == (Color::White, Color::Black) {
        color_pair.0 = Some(Color::Red);
        // colored_print!((Some(Color::Red), None), "\n{SHELL_ID} ");
    }
    // else {
    colored_print!((color_pair.0, color_pair.1), "\n{SHELL_ID} ");
    // }
}

// Halt the CPU until the next interrupt.
#[inline]
fn wait_for_interrupt() {
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack));
    }
}

pub fn shell_loop() -> ! {
    let mut paint_mode: bool;

    loop {
        paint_mode = false;
        hello_shell();

        // Inner loop: wait for Enter key
        loop {
            // Process all pending key events (there may be multiple if typing fast)
            let mut got_enter = false;

            while let Some(event) = get_next_key_event() {
                if !event.pressed {
                    continue; // Ignore key release events
                }

                match event.code {
                    KeyCode::Control(ControlKey::Enter) => {
                        got_enter = true;
                        break; // Exit the while loop
                    }
                    KeyCode::Char(c) => {
                        if event.modifiers == CTRL && c == '2' {
                            paint_mode = true;
                            got_enter = true; // Reuse flag to break outer loop
                            break;
                        }
                        print!("{c}");
                    }
                    KeyCode::Control(ControlKey::Backspace) => {
                        if !keyboard::input_buffer_empty() {
                            vga::WRITER.lock().delete_char();
                        }
                    }
                    _ => {}
                }
            }

            // If Enter was pressed, break out of the input loop
            if got_enter {
                break;
            }

            wait_for_interrupt();
        }

        // Process command or enter paint mode
        if paint_mode {
            crate::shell::paint::paint();
        } else {
            let input = keyboard::get_input_string();
            unsafe {
                SHELL.run_command(input);
            }
        }
        keyboard::clear_input();
    }
}

fn help(args: &[&str]) {
    unsafe {
        if args.is_empty() {
            println!("\nAvailable commands:");
            for cmd in &SHELL.commands[0..SHELL.command_count] {
                println!("  {} - {}", cmd.name, cmd.description);
            }
        } else {
            let command = args[0];
            for cmd in &SHELL.commands[0..SHELL.command_count] {
                if cmd.name == command {
                    println!("{} - {}", cmd.name, cmd.description);
                    return;
                }
            }
            println!("\nNo help available for '{}'", command);
        }
    }
}

// Initialize the shell
pub static mut SHELL: Shell = Shell::new();
