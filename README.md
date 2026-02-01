<p align="center">
  <img src="assets/app-icon.png" alt="Claude Usage Monitor" width="128" height="128">
</p>

# Claude Usage Monitor

A macOS menu bar app that displays your Claude AI usage limits at a glance.

<p align="center">
  <a href="https://github.com/Arielbs/claude-usage-monitor/releases/latest/download/Claude.Usage.Monitor_0.1.0_aarch64.dmg">
    <img src="https://img.shields.io/badge/Download-DMG-blue?style=for-the-badge&logo=apple" alt="Download DMG">
  </a>
</p>

<p align="center">
  <img src="assets/screenshot.png" alt="Screenshot" width="200">
</p>

## Features

- Real-time 5-hour and 7-day usage bars
- Countdown timers until limits reset
- Color-coded progress (green → yellow → red)
- Auto-detects your Chrome profile from Claude account
- Shows subscription tier (Max 5x, Max 20x, Pro, Free)
- Quick links to Claude.ai

## Installation

### Homebrew (recommended)

```bash
brew tap Arielbs/claude-usage-monitor https://github.com/Arielbs/claude-usage-monitor
brew install --cask claude-usage-monitor
```

### Manual

Download the `.dmg` from [Releases](https://github.com/Arielbs/claude-usage-monitor/releases) and drag to Applications.

> Requires [Claude Code](https://claude.ai/code) to be installed and logged in.

### Troubleshooting

If you see **"App is damaged and can't be opened"**, run this in Terminal:

```bash
xattr -cr "/Applications/Claude Usage Monitor.app"
```

This removes the quarantine flag that macOS adds to apps downloaded from the internet.

## Building

```bash
git clone https://github.com/Arielbs/claude-usage-monitor.git
cd claude-usage-monitor
npm install
cargo tauri build
```

## Author

**Ariel J. Ben-Sasson** — [@Arielbs](https://github.com/Arielbs)

Questions or feature requests? [Open an issue](https://github.com/Arielbs/claude-usage-monitor/issues)

## License

[MIT](LICENSE)
