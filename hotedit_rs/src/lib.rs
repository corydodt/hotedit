use git2;
use shlex;
use std::env;
use std::error::Error;
use std::fmt;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::Command;
use tempfile::{Builder, NamedTempFile};

const TEMP_EXT: &str = ".hotedit";
const EDITOR_FALLBACK: &str = "vi";

#[derive(Debug)]
struct UnchangedError;

impl Error for UnchangedError {}

impl fmt::Display for UnchangedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "editing operation did not change the contents")
    }
}

type EditorFindFn = fn() -> Result<String, Box<dyn Error>>;

// create a named tempfile and seed it with initial text
fn seed_tempfile(initial: &str) -> Result<NamedTempFile, Box<dyn Error>> {
    let mut ret = Builder::new().suffix(TEMP_EXT).tempfile()?;
    ret.write(initial.as_bytes())?;
    Ok(ret)
}

// return the contents of the tempfile and clean it up
fn harvest_tempfile(mut tf: NamedTempFile, persist: bool) -> Result<String, Box<dyn Error>> {
    let mut buffer = String::new();
    tf.seek(SeekFrom::Start(0))?;
    tf.read_to_string(&mut buffer)?;
    if persist {
        tf.keep()?;
    } else {
        tf.close()?;
    }
    Ok(buffer)
}

pub struct HotEdit<'he> {
    initial: &'he String,
    validate_unchanged: bool,
    delete_temp: bool,
    find_editor: EditorFindFn,
}

impl<'he> HotEdit<'he> {
    pub fn new(initial: &String) -> HotEdit {
        HotEdit {
            initial,
            validate_unchanged: false,
            delete_temp: true,
            find_editor: determine_editor,
        }
    }

    pub fn invoke(&self) -> Result<String, Box<dyn Error>> {
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

        let ret = harvest_tempfile(tf, !self.delete_temp)?;
        if self.validate_unchanged {
            if self.initial.eq(&ret) {
                return Err(Box::from(UnchangedError));
            }
        }
        Ok(ret)
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

fn determine_editor() -> Result<String, Box<dyn Error>> {
    if let Some(ret) = read_git_editor() {
        return Ok(ret);
    }
    if let Ok(ret) = env::var("EDITOR") {
        return Ok(ret);
    }
    if let Ok(ret) = env::var("VISUAL") {
        return Ok(ret);
    }
    Ok(String::from(EDITOR_FALLBACK))
}