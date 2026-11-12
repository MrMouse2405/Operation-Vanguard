# Maintainer installation and distribution

This is internal tooling; competitor mail assumes a supplied `vanguard` executable.

## Install the binary from Git

On a build machine with Rust/Cargo, a C toolchain (vendored Lua), and Bun on PATH:

```sh
cargo install --git https://github.com/MrMouse2405/Operation-Vanguard.git --locked --bin vanguard
```

Cargo runs the repository's `build.rs`: locked dashboard dependencies are installed,
static assets are built and embedded into the executable. No runtime Node/Bun server
is required. The Git revision must contain the source, Cargo.lock, ui/bun.lock and
frontend build configuration. Local uncommitted changes are not available from Git.

For local installation of the current checkout:

```sh
cargo install --path . --locked --bin vanguard
```

`VANGUARD_SKIP_UI_BUILD=1` is only for backend development with existing UI assets;
do not set it for distribution. Distribute the binary with the required topology
and allowed example agents. It still requires explicit `--script` and `--network`.

## Share VanMail

```sh
bun --cwd vandocs install --frozen-lockfile
bun --cwd vandocs run build
```

Share **vandocs/out/VanMail.html**. It contains all mail, highlighting, styles,
scripts and theme controls. Open it directly with a browser (`file://`); hash
navigation needs no server. Clipboard access depends on browser permissions;
select-and-copy remains available. Read/star/theme persistence is browser-local.
The distribution directory contains only `VanMail.html`; no asset folders or
separate lesson pages need to be distributed. `bun --cwd vandocs run preview`
serves this same file locally for an optional preview.
