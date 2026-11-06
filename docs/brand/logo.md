# Logo

A glossy chocolate ribbon shaped as a rounded diamond, on a transparent background.

| File | Use |
|---|---|
| `assets/brand/bourbon.png` | Master, 1024×1024, trimmed and centred. README header |
| `assets/icons/hicolor/<n>x<n>/apps/bourbon.png` | App icon at 16, 24, 32, 48, 64, 128, 256, 512. Installed to `/usr/share/icons/hicolor/` by the `.desktop`/AUR work (M7) |

The icon name `bourbon` matches the window app id, so Wayland compositors show it once the `.desktop` file is installed.

Regenerate the sizes from the master:

```bash
for n in 16 24 32 48 64 128 256 512; do
  magick assets/brand/bourbon.png -filter Lanczos -resize ${n}x${n} -strip assets/icons/hicolor/${n}x${n}/apps/bourbon.png
done
```
