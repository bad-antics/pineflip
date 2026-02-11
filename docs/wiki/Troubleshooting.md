# Troubleshooting

## Flipper not detected

1. Check USB cable (must support data, not charge-only)
2. Check udev rules (see Getting Started)
3. Check dmesg: `sudo dmesg | tail -20`
4. Try different USB port
5. Verify Flipper is on and not in DFU mode

## Screen mirroring laggy

- Reduce frame rate in Settings
- Use USB (not Bluetooth)
- Close other serial applications
- Check CPU usage — GTK4 rendering needs decent GPU

## File transfer fails

- Ensure SD card is not write-protected
- Check available space: PineFlip > Storage > Info
- Try smaller files first
- FAT32 has 4GB file size limit

## Build errors

```bash
# Missing GTK4
sudo apt install libgtk-4-dev

# Missing libadwaita
sudo apt install libadwaita-1-dev

# Rust too old
rustup update stable
```
