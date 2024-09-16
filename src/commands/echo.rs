pub fn run(args: &str) {
    let output : &str = args.get(5..).unwrap_or("");
    
    print!("\n{}", output);
}