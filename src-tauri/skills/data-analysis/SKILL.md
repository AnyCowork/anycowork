---
name: data-analysis
description: Perform data analysis and generate visualizations using Python (Pandas, Matplotlib, Seaborn). Supports CSV, JSON, and Excel formats.
license: MIT
---

# Data Analysis Skill

This skill enables deep data exploration, cleaning, aggregation, and visualization using Python's standard data science stack.

## Core Capabilities

1.  **Read Data**: Load data from CSV (`.csv`), Excel (`.xlsx`), JSON (`.json`), or Parquet files.
2.  **Clean & Transform**: Filter rows, handle missing values, reshape data (melt/pivot).
3.  **Aggregate**: Group by columns, calculate sums/means/counts or custom metrics.
4.  **Visualize**: Generate static plots (bar, line, scatter, histogram, boxplot, heatmap) using Matplotlib and Seaborn.

## Dependencies

*   `pandas` (pip install pandas openpyxl)
*   `matplotlib` (pip install matplotlib)
*   `seaborn` (pip install seaborn)

## Workflow Examples

### 1. Basic Statistical Summary

```python
import pandas as pd

# Load data
df = pd.read_csv('data.csv')

# Inspect basics
print(df.head())
print(df.info())
print(df.describe())

# Check for missing values
print(df.isnull().sum())
```

### 2. Visualization: Correlation Heatmap

```python
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

df = pd.read_csv('dataset.csv')
corr = df.corr()

plt.figure(figsize=(10, 8))
sns.heatmap(corr, annot=True, cmap='coolwarm')
plt.title('Correlation Matrix')
plt.savefig('correlation_heatmap.png')
```

### 3. Aggregation and Bar Chart

```python
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_excel('sales_data.xlsx')

# Group by Category and Sum Sales
category_sales = df.groupby('Category')['Sales'].sum().sort_values(ascending=False)

plt.figure(figsize=(12, 6))
category_sales.plot(kind='bar', color='skyblue')
plt.title('Total Sales by Category')
plt.xlabel('Category')
plt.ylabel('Sales ($)')
plt.tight_layout()
plt.savefig('sales_by_category.png')
```

## Best Practices

*   **Avoid large loops**: Use vectorized Pandas operations.
*   **Handle Errors**: Wrap file reading in try-except blocks.
*   **Visual Clarity**: Always label axes and titles in plots.
*   **File Formats**: Specify `engine='openpyxl'` for `.xlsx` files if needed.
