# Daily Note

A minimal desktop app for quickly appending notes to your Obsidian daily page.

![Daily Note screenshot](https://github.com/user-attachments/assets/20ad3a0c-86b2-4648-8df5-6992b17856d4)

## What it does

Press a shortcut, type a thought, hit Enter. Your note gets appended to your Obsidian daily note instantly and the window closes. Optionally prefix it with a timestamp.

## Requirements

- Obsidian CLI enabled
- Obsidian running in the background

## Installation

### From git 

```bash
cargo install --git https://github.com/1duxa/obsidian-quick-note --locked
```

### From source

```bash
git clone https://github.com/1duxa/obsidian-quick-note
cd obsidian-quick-note
cargo build --release
cp target/release/dn ~/.local/bin/
```

### Bind to a global shortcut

In your desktop environment, bind a keyboard shortcut (e.g. `Super+N`) to run:

```
/home/<you>/.local/bin/dn
```

## Usage

| Key | Action |
|---|---|
| `Enter` | Send note & close |
| `Shift+Enter` | New line |
| `Ctrl+Enter` | Toggle timestamp mode |
| `Esc` | Close without saving |

The colored bar at the bottom indicates the current mode:

- **Blue** — note is sent as-is
- **Red** — note is prefixed with the current time (`HH:MM`)

## Configuration

Config is saved automatically to:

- Linux: `~/.config/dailynote/config.ron` and to `~/.local/share/daily-note`
- macOS: `~/Library/Application Support/dailynote/config.ron`

The only persisted setting is the last-used mode (timestamp on/off).

## Building

```bash
cargo build --release
```

Dependencies are managed via Cargo. The font and icon are embedded at compile time — no assets need to be distributed alongside the binary.
