use hotedit::HotEdit;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("** not enough arguments");
        process::exit(1);
    }
    let hehe = HotEdit::new();
    match hehe.invoke(&args[1]) {
        Ok(edited) => {
            for line in edited.lines() {
                println!("> {}", line);
            }
        }
        Err(e) => {
            println!("** bad edit: {}", e);
            process::exit(1);
        }
    }
}
