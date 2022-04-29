# hotedit, in rust

An experimental implementation as an rlib (rust crate) and Python-importable
module. This works fine, but it was mostly a learning exercise for me.

## Build

1. Create a virtualenv, or you can safely use `poetry shell; poetry install` in the **parent**
directory.

2. Build with maturin
```
# installs into the virtualenv so you can play with it
maturin develop
# builds a manylinux wheel
maturin build
```

## Usage (Python)

```python
import hotedit
result = hotedit.invoke("this is my\neditable string\n", validate_unchanged=True)

# launches your editor; raises RuntimeError if you didn't change the text before
# closing it.
```

## Usage (Rust)

1. Add `hotedit` to Cargo.toml.

2. Code like the following:
```rust
use hotedit::HotEdit;

let hehe = HotEdit::new();
let edited_string = hehe.invoke("this is\nmy editable string\n").unwrap();

// n.b. I have not yet implemented a public interface to change the flags
// `delete_temp` or `validate_unchanged` in a Rust program, although this should be
// trivial.

// The option to implement your own `find_editor` Fn() is also not public.

// All of these flags and options do work in the Python cdylib interface, however,
// just like they do in the pure Python implementation.
```

See `src/bin/edit-string.rs` for a complete example.

## Notes/TODO

Compared to the Python API, this:

1. does not currently expose the Unchanged exception, I don't understand Rust exception
   hierarchies that well, so everything appears to be a RuntimeError.

2. Exposes `hotedit.invoke()` instead of `hotedit.hotedit()`. This is a minor implementation
   detail, probably fixable, having to do with function names inside `hoteditpy.rs`.

3. Exposes `def determine_editor()` in case you want to use that function to implement your own
   `find_editor` that upcalls `determine_editor()` as a last result. _BUT_, this version of 
   `determine_editor` does not take a default editor string as an argument. This didn't seem
   important enough to implement.

I haven't written any Rust tests for this, and there's been no attempt to make this pass the 
Python API's unit tests. The API is very similar, but the unit tests for pure-Python-hotedit
use mocks to patch internals which will not work in the context of a Rust
`cdylib`, so I would need to write a whole new test suite.
