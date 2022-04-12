# hotedit

A simple library for opening your user's favorite text editor

The `hotedit()` API looks at a few places in the shell environment for
a useful text editor, invokes it with your text, and finally returns
the edited result to your program (or raises an exception if an error 
occurred invoking the editor). 

## Install

```shell
pip install hotedit
```

### Installation for maintainers

After checking out github source, work with it using [Poetry].

1. Install `poetry`: 
  ```
  curl -sSL https://install.python-poetry.org | python3 -
  ```

2. Activate your poetry shell in the git repo you checked out, and install dependencies:
  ```
  poetry shell
  ...

  poetry install -E test
  ```

3. Run tests:
  ```
  make test
  ```

[poetry]: https://python-poetry.org

## Use

```python
# Get a string from the Internet so we can edit it:
import requests
URL = "https://pastebin.com/raw/Df9NAmYc"
response = requests.get(URL)

"""
Call `hotedit()` to launch the text editor to edit what was in the pastebin
document.

The first argument `initial` is the string to be shown in your editing buffer.
This creates a temp file, later deleted.

Other useful arguments:
  - validate_unchanged (default False): When True, raise `Unchanged` exception when
    the user did not make any edits.
  - delete_temp (default True): When False, doesn't delete the temp file (for 
    troubleshooting).
  - find_editor (default `hotedit.editor.determine_editor`): a zero-argument
    callable that returns the path to an editor executable (see below)
"""
from hotedit import hotedit
edited = hotedit(response.text)
# `edited` is the string returned from editing.

print("Your edited text:")
for line in edited.splitlines():
    print(f"> {line}")

# At this point we might upload the new string back to pastebin.
```

## How it determines your editor

Hotedit looks at your shell environment to determine which editor you're using.

It follows some very simple rules.

1. Check the following in order for a useful editor:
    1. `git config core.editor`
    1. `EDITOR` environment variable
    1. `VISUAL` environment variable

2. If one of these is a string, that string is returned as the path to an editor.

3. `hotedit()` itself will raise an exception if the path returned is not an
   editor executable, but it does not keep trying other options if this happens.

This means if `core.editor` is unset, `$EDITOR` is set to a path that's missing,
and `$VISUAL` is set to a valid path to an editor, `hotedit` will use `$EDITOR`
and fail, without checking `$VISUAL`.

### Overriding editor search with `find_editor`

If you don't like hotedit's search order, you can pass your own `find_editor`
function. Commonly, you want to try a different location first, then try the
default options. Here's an implementation of that:

```python
import os
import hotedit, hotedit.editor

def my_find_editor():
    if os.environ.get("MY_APP_TEXT_EDITOR"):
        return os.environ["MY_APP_TEXT_EDITOR"]
    return hotedit.editor.determine_editor()

edited = hotedit.hotedit("[MY_APP config]...", find_editor=my_find_editor)
...
```


## Suggestions for setting your editor

Many editor values _just work_, especially if they are commonly used in the
terminal. For example, you can just set `EDITOR=nano` or `EDITOR=vim`.

Editors that open in their own window, or make use of an already-open window,
almost always provide a command-line option for exactly the use cases
hotedit is used for. For example, try the following:

(Open a new gvim window, or open a tab in an existing gvim window, respectively):
```
EDITOR="gvim -f"  # or
EDITOR="gvim --remote-tab-wait"
```

(Open a tab in VS Code):
```
EDITOR="code -w"
```

If your users are likely to be using git, the best experience for them is usually
to invoke an editor the same way git would invoke one to ask for a commit
message. While it's not up to the `hotedit` library to tell you how to live your
life, here's how you would do that:

```
# VS code, for example
git config core.editor "code -w"
```

----

## Maintainer section: releasing

To cut a release of this software, automated tests must pass. Check under `Actions` for the latest commit.

#### Create an RC branch and test

- We use the Gitflow process. For a release, this means that you should have a v1.2.3-rc branch under your 
  develop branch. Like this:
  ```
    main  
    └── develop  
        └── v1.2.3-rc
  ```

- Update *this file*.
  
  1. Confirm that the docs make sense for the current release.
  1. Check links!
  1. Update the Changelog section at the bottom.

- Perform whatever tests are necessary.

#### Tag and cut the release with Github Actions

- Once you have tested in this branch, create a tag in the v1.2.3-rc branch:
  ```
  git tag -a -m v1.2.3 v1.2.3
  git push --tags
  ```

- Navigate to https://github.com/corydodt/hotedit/actions and run the action labeled `... release`.

    - You will be asked to choose a branch. Choose your rc branch, e.g. `v1.2.3-rc`

    - If you run this action without creating a tag on v1.2.3-rc first, the action will fail with an error and nothing will happen.

  If you have correctly tagged a commit and chosen the right branch, this will run and create a new release on the [Releases page].

- Edit the release on that page 

#### Merge up

- Finish up by merging your `-rc` branch into 
  1. `main` and then 
  2. `develop`.

## Changelog

<details><summary>(About: Keep-a-Changelog text format)</summary>

The format is based on [Keep a Changelog], and this project adheres to [Semantic
Versioning].
</details>

### versions [0.9.1]

- Initial public release.
- Automated builds, automated tests.


[Unreleased]: https://github.com/corydodt/hotedit/compare/v0.9.1..HEAD
[0.9.1]:        https://github.com/corydodt/hotedit/compare/v0.0..v0.9.1
[0.0]:        https://github.com/corydodt/hotedit/tree/v0.0


[latest release]: https://github.com/corydodt/hotedit/releases/latest

[Releases page]: https://github.com/corydodt/hotedit/releases

[Keep a Changelog]: https://keepachangelog.com/en/1.0.0/

[Semantic Versioning]: https://semver.org/spec/v2.0.0.html
