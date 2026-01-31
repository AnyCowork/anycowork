---
name: slack-gif-creator
description: Create animated GIFs optimized for Slack (small file size, loop, transparency) using Python libraries like imageio and Pillow.
license: MIT
---

# Slack GIF Creator Skill

This skill helps create lightweight, animated GIFs perfect for Slack reactions or messages using Python.

## Core Capabilities

1.  **Frame Generation**: Create frames programmatically using Pillow (PIL).
2.  **GIF Assembly**: Combine frames into animated GIFs using `imageio`.
3.  **Optimization**: Control FPS, duration, and color usage for small file sizes.

## Dependencies

*   `imageio` (pip install imageio) - GIF creation.
*   `Pillow` (pip install Pillow) - Frame drawing.

## Workflow Examples

### 1. Simple Growing Circle Animation

```python
import imageio
from PIL import Image, ImageDraw

frames = []
for i in range(20):
    img = Image.new('RGBA', (200, 200), (255, 255, 255, 0)) # Transparent background
    draw = ImageDraw.Draw(img)
    
    radius = 10 + i * 5
    center = (100, 100)
    msg = f"Frame {i}"
    
    draw.ellipse((center[0]-radius, center[1]-radius, center[0]+radius, center[1]+radius), fill='red')
    
    frames.append(img)

# Save as GIF
imageio.mimsave('growing_circle.gif', frames, fps=10, loop=0)
```

### 2. Text Animation (Marquee Style)

```python
import imageio
from PIL import Image, ImageDraw, ImageFont

frames = []
text = " LOADING... "
font = ImageFont.load_default() # Or load a TTF

for i in range(len(text)):
    img = Image.new('RGB', (200, 50), 'white')
    draw = ImageDraw.Draw(img)
    
    display_text = text[i:] + text[:i]
    draw.text((10, 15), display_text, fill='black', font=font)
    
    frames.append(img)

imageio.mimsave('marquee.gif', frames, fps=5, loop=0)
```

## Best Practices for Slack

*   **Size**: Keep under 5MB (ideally <1MB for quick loading).
*   **Dimensions**: Small (e.g., 128x128 for emojis, 300x300 for inline).
*   **FPS**: 10-15 FPS is usually sufficient for simple animations.
*   **Transparency**: Use `RGBA` mode for transparent backgrounds if needed.
