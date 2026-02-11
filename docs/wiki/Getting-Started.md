# Getting Started

## Install

### From Source
```bash
git clone https://github.com/bad-antics/pineflip
cd pineflip
cargo build --release
sudo make install
```

### Dependencies
```bash
# Debian/Ubuntu
sudo apt install libgtk-4-dev libadwaita-1-dev libudev-dev pkg-config

# Fedora
sudo dnf install gtk4-devel libadwaita-devel systemd-devel

# Arch
sudo pacman -S gtk4 libadwaita
```

## Connect Your Flipper

1. Connect Flipper Zero via USB
2. Launch PineFlip: `pineflip`
3. Flipper should auto-detect
4. If not, select port manually: Settings > Serial Port

## USB Permissions (Linux)

```bash
# Add udev rule for Flipper Zero
echo 'ATTRS{idVendor}=="0483", ATTRS{idProduct}=="5740", MODE="0666"' | \
  sudo tee /etc/udev/rules.d/42-flipperzero.rules
sudo udevadm control --reload-rules
```

## First Use

1. **Screen Mirror**: Click the monitor icon — see Flipper's screen on desktop
2. **Remote Control**: Use arrow keys or on-screen D-pad
3. **File Manager**: Click folder icon to browse SD card
4. **Screenshot**: Click camera icon to save PNG
