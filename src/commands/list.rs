pub fn run(mut parts : core::str::SplitWhitespace<'_>) {
    if let Some(option) = parts.next() {
        match option {
            "--help" => describe(),
            "--creator" => creator(),
            "--commands" => commands(),
            _ => describe(),
        }
    }
    else {
        describe();
    }
    
}

fn describe() {
    print!("\nhere is all the option :\n --creator\n --commands")
}

fn creator() {
    print!("\n Enzo Dal Cerro and Victor Portenseigne");
    print!("\n Edal--ce aka rust master and Viporten aka bbq master")
}

fn commands() {
    print!("\n list commands : \n\n echo\n clear\n list\n ft42\n custom \n");    
}