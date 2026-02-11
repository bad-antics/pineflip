# PineFlip CLI Reference

## Commands

```bash
# Connect to Flipper
pineflip connect [--port /dev/ttyACM0]

# Screenshot
pineflip screenshot [--output screenshot.png]

# File operations
pineflip ls /ext/badusb/
pineflip upload local-file.txt /ext/badusb/payload.txt
pineflip download /ext/subghz/capture.sub ./
pineflip mkdir /ext/my-folder
pineflip rm /ext/old-file.txt

# Device info
pineflip info
pineflip storage-info

# Screen recording
pineflip record --output flipper.gif --duration 10
```

## Scripting Example

```bash
#!/bin/bash
# Deploy all NullSec payloads to Flipper
pineflip connect
for f in payloads/*.txt; do
  pineflip upload "$f" "/ext/badusb/nullsec/$(basename $f)"
done
echo "Deployed $(ls payloads/*.txt | wc -l) payloads"
```
