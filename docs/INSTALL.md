# PineFlip Installation

## Build from Source
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libudev-dev pkg-config
git clone https://github.com/bad-antics/pineflip
cd pineflip
cargo build --release
sudo cp target/release/pineflip /usr/local/bin/
```

## Flatpak (coming soon)
```bash
flatpak install com.nullsec.PineFlip
```

## USB Setup
```bash
echo 'ATTRS{idVendor}=="0483", ATTRS{idProduct}=="5740", MODE="0666"' | \
  sudo tee /etc/udev/rules.d/42-flipperzero.rules
sudo udevadm control --reload-rules
```
