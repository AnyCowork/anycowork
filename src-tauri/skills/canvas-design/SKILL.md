---
name: canvas-design
description: Create programmatic digital art using Python libraries like Pillow (PIL), NumPy, and Matplotlib. Generate images, fractals, patterns, and abstract compositions.
license: MIT
---

# Canvas Design Skill

This skill enables the creation of generative art, patterns, and abstract compositions using Python.

## Core Capabilities

1.  **Generative Art**: Use algorithms to create unique visuals (fractals, noise, cellular automata).
2.  **Pattern Generation**: Create seamless tiles, geometric designs, and textures.
3.  **Image Manipulation**: Apply filters, blend modes, and transformations.
4.  **Drawing Primitives**: Render shapes, lines, and text using vector commands.

## Dependencies

*   `Pillow` (pip install Pillow) - Core image manipulation.
*   `numpy` (pip install numpy) - Mathematical operations for generative art.
*   `matplotlib` (pip install matplotlib) - Plotting complex forms.

## Workflow Examples

### 1. Simple Geometric Composition (Pillow)

```python
from PIL import Image, ImageDraw
import random

width, height = 800, 600
img = Image.new('RGB', (width, height), 'white')
draw = ImageDraw.Draw(img)

# Draw random circles
for _ in range(50):
    x = random.randint(0, width)
    y = random.randint(0, height)
    r = random.randint(10, 50)
    color = (random.randint(0, 255), random.randint(0, 255), random.randint(0, 255))
    draw.ellipse([x-r, y-r, x+r, y+r], fill=color, outline='black')

img.save('geometric_art.png')
```

### 2. Mandelbrot Fractal (NumPy + Matplotlib)

```python
import numpy as np
import matplotlib.pyplot as plt

def mandelbrot(h, w, max_iter=20):
    y, x = np.ogrid[-1.4:1.4:h*1j, -2:0.8:w*1j]
    c = x + y*1j
    z = c
    divtime = max_iter + np.zeros(z.shape, dtype=int)

    for i in range(max_iter):
        z = z**2 + c
        diverge = z*np.conj(z) > 2**2            
        div_now = diverge & (divtime == max_iter)  
        divtime[div_now] = i                  
        z[diverge] = 2                        

    return divtime

plt.figure(figsize=(10, 10))
plt.imshow(mandelbrot(800, 800, 50), cmap='magma')
plt.axis('off')
plt.savefig('mandelbrot.png', bbox_inches='tight', pad_inches=0)
```

## Best Practices

*   **Resolution**: Default to reasonable sizes like 800x600 or 1024x1024 for quick generation.
*   **Vector vs Raster**: Use `matplotlib` for complex curves/plots, `Pillow` for pixel manipulation.
*   **Performance**: Avoid extremely high iterations or resolution in Python loops.
