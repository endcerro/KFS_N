use crate::keyboard;
use crate::vga;
use crate::keyboard::*;
pub mod commands;
use commands::{*, paint};

const MAX_COMMANDS: usize = 20;
const _MAX_COMMAND_LENGTH: usize = 20;
const MAX_ARGS: usize = 10;
static SHELL_ID : &str = "Minishell > ";
#[derive(Clone, Copy)]
struct Command {
    name: &'static str,
    function: fn(&[&str]),
    description: &'static str, // Added for help functionality
}

pub struct Shell {
    commands: [Command; MAX_COMMANDS],
    command_count: usize,
}


impl Shell {
    const fn new() -> Self {
        Shell {
            commands: [Command { name: "", function: Shell::dummy_command, description: "" }; MAX_COMMANDS],
            command_count: 0,
        }
    }

    fn add_command(&mut self, name: &'static str, function: fn(&[&str]), description: &'static str) {
        if self.command_count < MAX_COMMANDS {
            self.commands[self.command_count] = Command { name, function, description };
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
            print!("\n{SHELL_ID}Unknown command: {}", command);
        }
    }

    fn dummy_command(_: &[&str]) {
        // This function should never be called
    }

}

pub fn init_shell() {
// In your init function or wherever appropriate:
    unsafe {
        SHELL.add_command("echo", echo::run, "Echo the input arguments");
        SHELL.add_command("clear", clear::run, "Clear the screen");
        SHELL.add_command("credits", credits::run, "Credits");
        SHELL.add_command("colors", custom::run, "Run a custom command");
        SHELL.add_command("ft", print_ft_42::run, "Print 42 logo");
        SHELL.add_command("stack", print_stack::run, "Print stack information");
        SHELL.add_command("help", help, "Display help information");
    }
}

pub fn hello_shell () {
    print!("\n{}", SHELL_ID);
}

pub fn shell_loop() -> ! {
    let mut paint_mode :bool;
    loop {
        paint_mode = false;
        hello_shell();
        loop {
        if let Some(event) = get_next_key_event() {
            if event.pressed == true {
                    // println!("{event}");
                match event.code {
                    KeyCode::Control(ControlKey::Enter) => break,
                    KeyCode::Char(c) =>
                    {
                        if event.modifiers == CTRL && c == '2'{
                            paint_mode = true;
                            break;
                        }
                        print!("{c}")
                    },
                    KeyCode::Control(ControlKey::Backspace) => {
                        if !keyboard::input_buffer_empty() {
                            vga::WRITER.lock().delete_char();
                        }},
                    _ => ()}}}}
        if paint_mode {
            crate::shell::paint::paint();
        } else {
            let len = keyboard::get_input_string();
            unsafe {
                SHELL.run_command(len);
            }
        }

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
