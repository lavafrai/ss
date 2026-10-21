# ss

Choose an SSH host from an interactive, searchable terminal menu.

Run `ss`, type to search, and press Enter to connect. Use arrow keys or the mouse
to select a host; Escape cancels. Connections use your installed SSH client in
the current terminal.

Reads `~/.ssh/config` (`%USERPROFILE%\.ssh\config` on Windows), including
`Include` files. Wildcard patterns and negated entries are omitted.
This lists configured aliases; it does not check connectivity.
Unknown directives are ignored; `Match` conditions are not evaluated.
Aliases are sorted and deduplicated. An interactive terminal is required.

SSH must be installed and available through `PATH`. Connections use the original
alias so the SSH client applies the full configuration, including `Match`, keys,
jump hosts, and forwarding. The displayed address is only a preview.

## Build and run

```sh
cargo build --release
cargo run --release
```

The executable is `target/release/ss` on macOS/Linux and
`target\release\ss.exe` on Windows. Build on the target operating system.
