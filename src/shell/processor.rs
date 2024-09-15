use crate::commands;

static SHELL_ID : &str = "Minishell > ";

pub fn hello_shell () {
    print!("\n{}", SHELL_ID);
}

pub fn process_command(input: &str) {
    let mut parts = input.split_whitespace();
    if let Some(command) = parts.next() {
        match command {
            "echo" => commands::echo::run(input),
            _ => print!("\n{SHELL_ID}Unknown command: {}", command),
        }
    }
}