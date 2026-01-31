---
name: xlsx
description: Create, read, and modify Excel (.xlsx) spreadsheets using Python libraries openpyxl and pandas.
license: Apache 2.0
---

# XLSX Skill

This skill provides capabilities to work with Excel spreadsheets (.xlsx) using Python.

## Core Capabilities

1.  **Create New Spreadsheets**: Generate .xlsx files programmatically.
2.  **Read Data**: Extract data from sheets into structured formats.
3.  **Modify Existing Sheets**: Update cells, add rows, or change formatting.
4.  **Data Analysis**: Load data into pandas DataFrames for analysis.

## Dependencies

This skill relies on `openpyxl` and `pandas`.

```bash
pip install openpyxl pandas
```

## Workflows

### 1. Creating a New Workbook

Use `openpyxl` to create a new workbook.

```python
from openpyxl import Workbook

wb = Workbook()
ws = wb.active
ws.title = "My Sheet"

# Add data
ws['A1'] = "Name"
ws['B1'] = "Value"
ws.append(["Alice", 100])
ws.append(["Bob", 200])

wb.save("output.xlsx")
```

### 2. Reading with Pandas

To easily read an Excel file into a DataFrame:

```python
import pandas as pd

df = pd.read_excel("data.xlsx", sheet_name="Sheet1")
print(df.head())
```

### 3. Modifying an Existing Workbook

To edit an existing file:

```python
from openpyxl import load_workbook

wb = load_workbook("existing.xlsx")
ws = wb.active

# Update a cell
ws['B2'] = 999

# save
wb.save("updated.xlsx")
```

## Best Practices

-   **Library Choice**: Use `pandas` for heavy data reading/analysis. Use `openpyxl` for formatting, styles, and precise cell manipulation.
-   **File Safety**: Close files properly or use context managers if available (though `openpyxl` handles save/close explicitly).
-   **Large Files**: For very large files, consider using `openpyxl`'s `read_only=True` mode or processing in chunks with `pandas`.

## Resources

-   [openpyxl Documentation](https://openpyxl.readthedocs.io/en/stable/)
-   [pandas read_excel Documentation](https://pandas.pydata.org/docs/reference/api/pandas.read_excel.html)
