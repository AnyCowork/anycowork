---
name: docx
description: Create, read, and modify Microsoft Word (.docx) documents using Python and standard libraries.
license: MIT
---

# DOCX Skill

This skill provides capabilities to work with Microsoft Word documents (.docx) using Python.

## Core Capabilities

1.  **Create New Documents**: Generate professional .docx files programmatically.
2.  **Read Content**: Extract text and structure from existing .docx files.
3.  **Modify Existing Documents**: Append text, replace content, or update formatting.

## Dependencies

This skill relies on the `python-docx` library.
To install: `pip install python-docx`

## Workflows

### 1. Creating a New Document

Use `python-docx` to create a new document.

```python
from docx import Document
from docx.shared import Pt

# Initialize
doc = Document()

# Add Title
doc.add_heading('Document Title', 0)

# Add Paragraph
p = doc.add_paragraph('This is a paragraph.')
p.add_run(' bold text').bold = True
p.add_run(' and some ')
p.add_run('italic text.').italic = True

# Add Image (if available)
# doc.add_picture('image.png', width=Inches(1.25))

# Save
doc.save('output.docx')
```

### 2. Reading a Document

To extract specific content or read paragraphs:

```python
from docx import Document

doc = Document('input.docx')

full_text = []
for para in doc.paragraphs:
    full_text.append(para.text)

print('\n'.join(full_text))
```

### 3. Modifying an Existing Document

To edit an existing file (e.g., replace placeholder text):

```python
from docx import Document

doc = Document('template.docx')

for paragraph in doc.paragraphs:
    if 'PLACEHOLDER' in paragraph.text:
        paragraph.text = paragraph.text.replace('PLACEHOLDER', 'Replacement Text')

doc.save('updated.docx')
```

## Best Practices

*   **Structure**: Use headings (level 1-3) to organize content clearly.
*   **Formatting**: Use styles (`Normal`, `Heading 1`, `List Bullet`) rather than direct formatting when possible.
*   **Images**: Ensure images are accessible in the working directory before adding them.
*   **Validation**: Always check if the file exists before reading.
