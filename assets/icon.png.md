# Converting ShitRust Icon from SVG to PNG

To convert the ShitRust SVG icon to PNG format, you can use one of the following methods:

## Method 1: Using Inkscape (Recommended)

1. Install [Inkscape](https://inkscape.org/) if you don't have it already
2. Open the SVG file in Inkscape
3. Go to File > Export PNG Image
4. Set the desired export size (recommended: 512x512)
5. Click "Export As..." and save the file as `icon.png`

## Method 2: Using ImageMagick

```bash
# Install ImageMagick first
magick convert assets/icon.svg -resize 512x512 assets/icon.png
```

## Method 3: Using browser

1. Open the SVG in a modern web browser
2. Right-click and select "Save image as..."
3. Choose PNG format and save as `icon.png`

## Method 4: Online converters

You can use online tools like:
- [Convertio](https://convertio.co/svg-png/)
- [CloudConvert](https://cloudconvert.com/svg-to-png)

## File Information

The PNG file should have the following properties:
- Size: 512x512 pixels
- Format: PNG with transparency
- Location: Place the PNG file in the `assets` directory

Also consider generating different sizes for various platforms:
- 16x16, 32x32, 48x48 for Windows .ico files
- 16x16 to 1024x1024 for macOS .icns files 