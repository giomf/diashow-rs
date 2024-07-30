# Command-Line Help for `diashow-rs`

This document contains the help content for the `diashow-rs` command-line program.

**Command Overview:**

* [`diashow-rs`↴](#diashow-rs)
* [`diashow-rs start`↴](#diashow-rs-start)

## `diashow-rs`

Fotobox diashow

**Usage:** `diashow-rs <COMMAND>`

###### **Subcommands:**

* `start` — Start the diashow



## `diashow-rs start`

Start the diashow

**Usage:** `diashow-rs start [OPTIONS] --images <IMAGES> --duration <DURATION>`

###### **Options:**

* `--images <IMAGES>` — Folder where to search for images
* `--duration <DURATION>` — Duration that one image is displayed in seconds
* `--start-index <START_INDEX>` — Index where to start. A negative number will start at the end
* `--fade-iteration-duration <FADE_ITERATION_DURATION>` — Duration of one fade iteration in miliseconds
* `--fade-iteration-step <FADE_ITERATION_STEP>` — Step size of one fade iteration



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
