//! # The HotEdit crate

use std::env;
use std::error::Error;
use std::fmt;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::Command;
use tempfile::{Builder, NamedTempFile};

mod hoteditpy;

const TEMP_EXT: &str = ".hotedit";
const EDITOR_FALLBACK: &str = "vi";

/// An editing operation that didn't change the input text
///
/// This may optionally be considered an error by the invoking
/// program.
#[derive(Debug)]
pub struct UnchangedError;

impl Error for UnchangedError {}

impl fmt::Display for UnchangedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "editing operation did not change the contents")
    }
}

/// zero-arg function to find an editor command and return it
pub type ResultString = Result<String, Box<dyn Error>>;
pub type EditorFindFn = fn() -> ResultString;

/// create a named tempfile and seed it with initial text
fn seed_tempfile(initial: &str) -> Result<NamedTempFile, Box<dyn Error>> {
    let mut ret = Builder::new().suffix(TEMP_EXT).tempfile()?;
    ret.write_all(initial.as_bytes())?;
    Ok(ret)
}

/// return the contents of the tempfile and clean it up
///
/// With `persist`, at the end of the operation, keep the file instead of deleting.
fn harvest_tempfile(mut tf: NamedTempFile, persist: bool) -> ResultString {
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

/// A HotEdit operation
///
///   1. Search predefined places for an editor
///   2. Launch the editor with the caller's specified initial text in a buffer
///   3. Wait for user to edit
///   4. Return the new text to the caller
///   5. (optionally) Delete the temp file.
pub struct HotEdit {
    validate_unchanged: bool,
    delete_temp: bool,
    find_editor: EditorFindFn,
}

impl HotEdit {
    pub fn new() -> HotEdit {
        HotEdit {
            validate_unchanged: false,
            delete_temp: true,
            find_editor: determine_editor,
        }
    }

    /// Invoke the hotedit operation; launches the editor with initial text
    pub fn invoke(&self, initial: &String) -> ResultString {
        let editor = (self.find_editor)()?;
        let mut argv = match shlex::split(&editor) {
            Some(r) => r,
            None => return Err(Box::from("couldn't split editor args")),
        };
        if argv.is_empty() {
            return Err(Box::from("empty command string"));
        }

        let mut cmd = Command::new(argv.remove(0));

        let tf = seed_tempfile(initial)?;
        argv.push(tf.path().to_str().unwrap().to_owned());

        cmd.args(argv);
        cmd.status()?;

        let ret = harvest_tempfile(tf, !self.delete_temp)?;
        if self.validate_unchanged && initial.eq(&ret) {
            return Err(Box::from(UnchangedError));
        }
        Ok(ret)
    }
}

impl Default for HotEdit {
    fn default() -> Self {
        Self::new()
    }
}

/// Try to get an editor from git config
fn read_git_editor() -> Option<String> {
    let cfg = match git2::Config::open_default() {
        Ok(c) => c,
        Err(_) => return None,
    };

    let ret = match cfg.get_entry("core.editor") {
        Ok(editor) => editor.value().map(String::from),
        Err(_) => None,
    };
    ret
}

/// Inspect git core.editor setting, $EDITOR and $VISUAL for a command that opens an editor
///
/// If no editor is found, open vi
pub fn determine_editor() -> ResultString {
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
