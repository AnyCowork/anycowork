---
name: pdf
description: PDF manipulation tasks such as reading text, extracting metadata, merging, splitting, and rotating pages using the Python pypdf library.
license: Apache 2.0
---

# PDF Manipulation Skill

This skill allows you to perform various operations on PDF files using the `pypdf` library in Python.

## Core Capabilities

1.  **Read Text**: Extract text content from PDF pages.
2.  **Extract Metadata**: Read document information (author, title, etc.).
3.  **Merge PDFs**: Combine multiple PDF files into one.
4.  **Split PDFs**: Select specific pages to save as a new PDF.
5.  **Rotate Pages**: Change page orientation.

## Dependencies

This skill relies on the `pypdf` library and its dependencies.

```bash
pip install pypdf
```

## Workflows

### 1. Extracting Text from a PDF

To read text from a PDF file:

```python
from pypdf import PdfReader

reader = PdfReader("example.pdf")
number_of_pages = len(reader.pages)
page = reader.pages[0]
text = page.extract_text()
print(text)
```

### 2. Merging PDFs

To merge multiple PDFs into a single file:

```python
from pypdf import PdfWriter

merger = PdfWriter()

for pdf in ["file1.pdf", "file2.pdf", "file3.pdf"]:
    merger.append(pdf)

merger.write("merged-pdf.pdf")
merger.close()
```

### 3. Extracting Specific Pages

To extract specific pages (e.g., pages 1 and 3 - 0-indexed) into a new file:

```python
from pypdf import PdfReader, PdfWriter

reader = PdfReader("source.pdf")
writer = PdfWriter()

# Add page 1 (index 0) and page 3 (index 2)
writer.add_page(reader.pages[0])
writer.add_page(reader.pages[2])

with open("extracted_pages.pdf", "wb") as f:
    writer.write(f)
```

### 4. Reading Metadata

To access PDF metadata:

```python
from pypdf import PdfReader

reader = PdfReader("example.pdf")
meta = reader.metadata

print(f"Title: {meta.title}")
print(f"Author: {meta.author}")
print(f"Producer: {meta.producer}")
```

## Best Practices

-   **File Paths**: Always use absolute paths or verify the current working directory.
-   **Error Handling**: Wrap operations in try/except blocks to handle `FileNotFoundError` or encrypted/corrupted PDFs.
-   **Encryption**: `pypdf` can handle encrypted PDFs if the password is known using `reader.decrypt('password')`.

## Resources

-   [pypdf Documentation](https://pypdf.readthedocs.io/en/stable/)
