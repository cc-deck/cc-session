# Quickstart: Release, Documentation & Demo

## Setting Up a Release

```bash
# One-time: initialize cargo-dist
cargo dist init

# Tag and release
git tag v0.1.0
git push --tags
# GitHub Actions builds binaries and updates Homebrew tap
```

## Regenerating the Demo GIF

```bash
# Prerequisites: asciinema, agg, tmux
cd demo/
./record-demo.sh    # generates demo.cast
./export-gif.sh     # converts to demo.gif
```

## Installing (end user)

```bash
# Homebrew
brew install rhuss/tap/cc-session

# Install script
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/rhuss/cc-session/releases/latest/download/cc-session-installer.sh | sh

# From source
cargo install --git https://github.com/rhuss/cc-session
```
