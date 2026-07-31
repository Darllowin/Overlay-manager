# overlay-manager

TUI application for managing Gentoo Portage overlays.

![Rust](https://img.shields.io/badge/rust-1.95%2B-orange.svg)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Browse, search, add, remove, and sync Gentoo overlays from a fast terminal interface built with [ratatui](https://github.com/ratatui/ratatui).

## Features

- **Browse** available overlays from the [official Gentoo registry](https://api.gentoo.org/overlays/repositories.xml), GitHub, and your custom bookmarks
- **Search** overlays and packages with real-time filtering
- **Install/uninstall** overlays with a single keystroke — writes to `/etc/portage/repos.conf/`
- **Sync** a single overlay (`emaint sync -r`) or all at once (`emaint sync -a`)
- **View all packages** an overlay provides — scans ebuild files in `/var/db/repos/<name>/`
- **Package descriptions** parsed from `metadata.xml` and ebuild `DESCRIPTION`
- **Confirmation dialogs** for destructive actions
- **Animated spinners** during sync and cache refresh
- **Auto-elevation** via `doas` or `sudo` at startup
- **Localization**: English and Russian based on `$LANG`

## Scheme

```
┌─ Browse ─── Installed ──── overlay-manager ─────────────────────────────────┐
│ /guru ▌                                                                     │
│ [R] [✓] guru                     │  guru  ✓ installed                       │
│ [G]     brave-overlay            │  GURU - Gentoo User Repository           │
│ [C]     my-local                 │                                          │
│ ...                              │  Source: github.com/gentoo-mirror/guru   │
│                                  │  Owner: guru@gentoo.org                  │
│                                  │  Status: experimental / unofficial       │
│  Available (455)                 │  Packages: 12 pkg(s)                     │
├──────────────────────────────────┴──────────────────────────────────────────┤
│  a:add+sync  S:sync all  d:rm  /:search  r:refresh  h:help  q:quit         │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Installation

### From source

```bash
git clone https://github.com/Darllowin/Overlay-manager.git
cd overlay-manager
cargo build --release
sudo cp target/release/overlay-manager /usr/local/bin/
```

### Requirements

- Rust 1.95+
- `doas` or `sudo` for write operations
- `emaint` (part of `sys-apps/portage`) for syncing

## Usage

```bash
overlay-manager        # launch TUI (auto-elevates via doas/sudo if needed)
```

### Keybindings

| Key | Action |
|-----|--------|
| `j`/`k`/`↑`/`↓` | Navigate list |
| `g`/`G` | Top / bottom |
| `Tab` | Browse ↔ Installed tabs |
| `/` | Search / filter |
| `a` | Add overlay + sync |
| `S` | Sync all overlays |
| `d` | Remove overlay (config + files, with confirmation) |
| `r` | Refresh overlay cache |
| `Enter` | View all packages in selected overlay (with search and descriptions) |
| `h` | Help |
| `q` | Quit |

## License

MIT
