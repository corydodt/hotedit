//! # The HotEdit crate

use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
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
pub type EditorFindFn = dyn Fn() -> ResultString;

/// create a named tempfile and seed it with initial text
fn seed_tempfile(initial: &str) -> Result<NamedTempFile, Box<dyn Error>> {
    let mut ret = Builder::new().suffix(TEMP_EXT).tempfile()?;
    ret.write_all(initial.as_bytes())?;
    Ok(ret)
}

/// clean up the temp file
///
/// With `persist`, at the end of the operation, keep the file instead of deleting.
fn harvest_tempfile(tf: NamedTempFile, persist: bool) -> Result<(), Box<dyn Error>> {
    if persist {
        tf.keep()?;
    } else {
        tf.close()?;
    }
    Ok(())
}

/// A HotEdit operation
///
///   1. Search predefined places for an editor
///   2. Launch the editor with the caller's specified initial text in a buffer
///   3. Wait for user to edit
///   4. Return the new text to the caller
///   5. (optionally) Delete the temp file.
pub struct HotEdit<'he> {
    validate_unchanged: bool,
    delete_temp: bool,
    find_editor: &'he EditorFindFn,
}

impl<'he> HotEdit<'he> {
    pub fn new() -> HotEdit<'he> {
        HotEdit {
            validate_unchanged: false,
            delete_temp: true,
            find_editor: &determine_editor,
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

        // Some edit operations, but not all, edit the file in-place, while
        // others do an atomic replace. To handle either case, reopen the file
        // after the write has occurred so we're not reading from a stale inode.
        let mut tf2 = File::open(tf.path())?;
        let mut buffer = String::new();
        tf2.read_to_string(&mut buffer)?;

        // clean up
        harvest_tempfile(tf, !self.delete_temp)?;

        if self.validate_unchanged && initial.eq(&buffer) {
            return Err(Box::from(UnchangedError));
        }
        Ok(buffer)
    }
}

impl<'he> Default for HotEdit<'he> {
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
