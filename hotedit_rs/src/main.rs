use git2;
use shlex;
use std::env;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process;
use std::process::Command;
use tempfile;

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

impl<'he> HotEdit<'he> {
    fn new(initial: &String) -> HotEdit {
        HotEdit {
            initial,
            validate_unchanged: false,
            delete_temp: true,
            find_editor: determine_editor,
        }
    }

    fn invoke(&self) -> Result<String, &'static str> {
        let editor = (self.find_editor)();
        let editor = match editor {
            Ok(s) => s,
            Err(e) => {
                return Err(e);
            }
        };
        let mut argv = if let Some(r) = shlex::split(&editor) {
            r
        } else {
            return Err("couldn't split editor args");
        };
        if argv.len() < 1 {
            return Err("empty command string");
        }

        let mut cmd: Command;
        cmd = Command::new(argv.remove(0));

        // inject intial into a temp file
        let mut tf = if let Ok(r) = tempfile::NamedTempFile::new() {
            r
        } else {
            return Err("couldn't create temp file");
        };

        // println!("TEMP {}", tf.path().to_str().unwrap());

        tf.write(self.initial.as_bytes()).unwrap();
        argv.push(tf.path().to_str().unwrap().to_owned());

        cmd.args(argv);
        if let Err(_) = cmd.status() {
            return Err("editor exited with an error (rc != 0)");
        }

        // read from temp file
        if let Err(_) = tf.seek(SeekFrom::Start(0)) {
            return Err("couldn't seek in tempfile");
        }
        let mut buffer = Vec::new();
        if let Err(_) = tf.read_to_end(&mut buffer) {
            return Err("couldn't read tempfile after save");
        }
        if let Err(_) = tf.close() {
            return Err("couldn't delete tempfile");
        }
        match String::from_utf8(buffer) {
            Ok(s) => Ok(s),
            Err(_) => Err("couldn't convert edited file as utf-8"),
        }
    }
}

fn read_git_editor() -> Option<String> {
    let cfg = match git2::Config::open_default() {
        Ok(c) => c,
        Err(_) => {
            return None;
        }
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
