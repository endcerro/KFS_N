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
            "clear" => commands::clear::run(),
            "list" => commands::list::run(parts),
            "custom" => commands::custom::run(parts),
            "ft42" => commands::print_ft_42::run(),
            "stack" => commands::print_stack::run(),
            _ => print!("\n{SHELL_ID}Unknown command: {}", command),
        }
    }
}