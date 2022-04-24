use git2;
use shlex;
use std::env;
use std::error::Error;
use std::io::{Read, Write};
use std::process;
use std::process::Command;
use tempfile::NamedTempFile;

// const TEMP_EXT: &str = ".hotedit";
// const EDITOR_FALLBACK: &str = "vi";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("** not enough arguments");
        process::exit(1);
    }
    let hehe = HotEdit::new(&args[1]);
    match hehe.invoke() {
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

type EditorFindFn = fn() -> Result<String, &'static str>;

struct HotEdit<'he> {
    initial: &'he String,
    validate_unchanged: bool,
    delete_temp: bool,
    find_editor: EditorFindFn,
}

// create a named tempfile and seed it with initial text
fn seed_tempfile(initial: &str) -> Result<NamedTempFile, Box<dyn Error>> {
    let mut ret = match NamedTempFile::new() {
        Ok(x) => x,
        Err(_) => return Err(Box::from("Couldn't create tempfile")),
    };
    ret.write(initial.as_bytes())?;
    Ok(ret)
}

// return the contents of the tempfile and clean it up
fn harvest_tempfile<'a>(mut tf: NamedTempFile) -> Result<String, Box<dyn Error>> {
    let mut buffer = String::new();
    let _ = tf.read_to_string(&mut buffer)?;
    tf.close()?;
    Ok(buffer)
}

impl<'he> HotEdit<'he> {
    fn new(initial: &String) -> HotEdit {
        HotEdit {
            initial,
            validate_unchanged: false,
            delete_temp: true,
            find_editor: determine_editor,
        }
    }

    fn invoke(&self) -> Result<String, Box<dyn Error>> {
        let editor = (self.find_editor)()?;
        let mut argv = match shlex::split(&editor) {
            Some(r) => r,
            None => return Err(Box::from("couldn't split editor args")),
        };
        if argv.len() < 1 {
            return Err(Box::from("empty command string"));
        }

        let mut cmd: Command;
        cmd = Command::new(argv.remove(0));

        let tf = seed_tempfile(self.initial)?;
        argv.push(tf.path().to_str().unwrap().to_owned());

        cmd.args(argv);
        cmd.status()?;

        harvest_tempfile(tf)
    }
}

fn read_git_editor() -> Option<String> {
    let cfg = match git2::Config::open_default() {
        Ok(c) => c,
        Err(_) => return None,
    };

    let ret = match cfg.get_entry("core.editor") {
        Ok(editor) => {
            if let Some(v) = editor.value() {
                Some(String::from(v))
            } else {
                None
            }
        }
        Err(_) => None,
    };
    ret
}

fn determine_editor(/* TODO fallback */) -> Result<String, &'static str> {
    if let Some(ret) = read_git_editor() {
        return Ok(ret);
    }
    if let Ok(ret) = env::var("EDITOR") {
        return Ok(ret);
    }
    if let Ok(ret) = env::var("VISUAL") {
        return Ok(ret);
    }
    // TODO fallback
    Err("No editor found (checked git, $EDITOR and $VISUAL)")
}
