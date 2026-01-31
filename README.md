# Claude Usage Monitor

A lightweight macOS menu bar app that shows your Claude AI usage limits at a glance.

## Features

- **Real-time usage display** - See your 5-hour and 7-day Claude usage limits
- **Visual progress bars** - Color-coded (green/yellow/red) based on usage level
- **Countdown timers** - Know exactly when your limits reset
- **Menu bar integration** - Lives in your menu bar, shows current percentage
- **Quick links** - One-click access to Claude.ai home and settings
- **Multi-account support** - Select which Chrome profile to use for opening links

## Requirements

- macOS 10.15 or later
- [Claude Code](https://claude.ai/code) must be installed and logged in

> **Note:** This app reads your Claude OAuth token from the macOS Keychain (stored by Claude Code). It's a companion app for Claude Code users.

## Installation

### Homebrew (coming soon)
```bash
brew install --cask claude-usage-monitor
```

### Manual Download
Download the latest `.dmg` from [Releases](https://github.com/Arielbs/claude-usage-monitor/releases), open it, and drag to Applications.

## How It Works

The app reads your Claude OAuth token from the macOS Keychain (stored by Claude Code) and polls the Anthropic usage API every 60 seconds to fetch your current limits.

## Building from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI
cargo install tauri-cli

# Clone and build
git clone https://github.com/Arielbs/claude-usage-monitor.git
cd claude-usage-monitor
cargo tauri build
```

The built app will be at `src-tauri/target/release/bundle/macos/Claude Usage Monitor.app`

## Privacy

- All data stays local on your machine
- Credentials are read from your local Keychain only
- The only external request is to Anthropic's API to fetch your usage data
- No analytics or telemetry

## Tech Stack

- [Tauri 2](https://tauri.app/) - Rust-based desktop app framework
- Vanilla HTML/CSS/JS - Lightweight frontend
- macOS Keychain - Secure credential storage

## License

[MIT](LICENSE)
