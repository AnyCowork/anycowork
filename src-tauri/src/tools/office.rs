use super::{Tool, ToolContext};
use crate::permissions::{PermissionRequest, PermissionType};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
// CSV
use csv::ReaderBuilder;

// Excel
use calamine::{open_workbook, Data, Reader, Xlsx};

// PDF
// Note: pdf-extract might panic or fail if deps are missing, wrapping carefully
use docx_rs::*;
use pdf_extract::extract_text;
use std::fs::File; // Needed for docx writing

// Docx
// docx-rs is mainly for writing. For simple reading, let's use a zip extractor approach
// akin to 'zip' crate if we wanted to be pure, but since we added docx-rs, let's see if we can use it.
// Actually, docx-rs 0.4.x is indeed mostly for writing.
// A robust way to read docx text without external deps (other than zip) is to unzip and read document.xml.
// Since we didn't add 'zip' crate, we might be limited.
// However, let's try to use a simple text extraction if possible or just implement a basic CSV/Excel/PDF first.
// Wait, I can add 'zip' crate or 'xml-rs' if I really need to read docx effectively if docx-rs doesn't support reading.
// For now, let's implement the others and stub DOCX or use a simple string search if it's text based (it isn't, it's zip).
// UPDATE: I will assume I can add 'zip' if needed, but for now let's implement what we can.
// Actually, let's check if 'docx-rs' has read capability. It has `read_docx` in newer versions but it might be limited.
// Let's implement CSV, Excel, PDF first, and separate DOCX if complex.

use tauri::Runtime;

pub struct OfficeTool;

#[async_trait]
impl<R: Runtime> Tool<R> for OfficeTool {
    fn name(&self) -> &str {
        "office"
    }

    fn description(&self) -> &str {
        "Read content from Office files (Excel, CSV, PDF, Word). Supported formats: .xlsx, .csv, .pdf"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["read_excel", "read_csv", "read_pdf", "write_docx"],
                    "description": "The operation to perform"
                },
                "path": {
                    "type": "string",
                    "description": "Relative path to the file"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write (for write_docx)"
                }
            },
            "required": ["operation", "path"]
        })
    }

    async fn validate_args(&self, args: &Value) -> Result<(), String> {
        let path_str = args["path"].as_str().ok_or("Missing path")?;
        if path_str.contains("..") || path_str.starts_with("/") {
            return Err("Access denied: Paths must be relative".to_string());
        }
        Ok(())
    }

    fn needs_summarization(&self, _args: &Value, _result: &Value) -> bool {
        true // Always summarize office file content reading
    }

    async fn execute(&self, args: Value, ctx: &ToolContext<R>) -> Result<Value, String> {
        let op = args["operation"].as_str().ok_or("Missing operation")?;
        let path_str = args["path"].as_str().ok_or("Missing path")?;

        // Permission check
        let permission_type = if op == "write_docx" {
            PermissionType::FilesystemWrite
        } else {
            PermissionType::FilesystemRead
        };

        let perm_req = PermissionRequest {
            id: uuid::Uuid::new_v4().to_string(),
            permission_type,
            message: format!(
                "Agent wants to {} office file at {}",
                op.replace("_", " "),
                path_str
            ),
            metadata: {
                let mut map = HashMap::new();
                map.insert("operation".to_string(), op.to_string());
                map.insert("path".to_string(), path_str.to_string());
                map.insert("resource".to_string(), path_str.to_string());
                map
            },
        };

        if !ctx
            .permissions
            .request_permission(ctx.window.as_ref(), perm_req)
            .await?
        {
            return Err("Permission denied".to_string());
        }

        let root = std::env::current_dir().unwrap_or(PathBuf::from("."));
        let target_path = root.join(path_str);

        if !target_path.exists() {
            return Err("File not found".to_string());
        }

        match op {
            "read_csv" => self.read_csv(&target_path),
            "read_excel" => self.read_excel(&target_path),
            "read_pdf" => self.read_pdf(&target_path),
            "write_docx" => self.write_docx(&target_path, &args),
            _ => Err(format!("Unknown operation: {}", op)),
        }
    }
}

impl OfficeTool {
    fn write_docx(&self, path: &PathBuf, args: &Value) -> Result<Value, String> {
        let content = args["content"]
            .as_str()
            .ok_or("Missing content for write_docx")?;

        // Ensure parent dir exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let file = File::create(path).map_err(|e| e.to_string())?;

        let mut doc = Docx::new();
        // Split by newlines to create paragraphs
        for line in content.split('\n') {
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(line)));
        }

        doc.build()
            .pack(file)
            .map_err(|e| format!("Failed to write docx: {}", e))?;

        Ok(json!("Document created successfully"))
    }
    fn read_csv(&self, path: &PathBuf) -> Result<Value, String> {
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)
            .map_err(|e| format!("Failed to open CSV: {}", e))?;

        let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
        let mut rows = Vec::new();

        for result in rdr.records() {
            let record = result.map_err(|e| e.to_string())?;
            let mut row_map = HashMap::new();
            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    row_map.insert(header.to_string(), field.to_string());
                }
            }
            rows.push(row_map);
            // Limit to 1000 rows to avoid blowing up memory/token context
            if rows.len() >= 1000 {
                break;
            }
        }

        Ok(json!(rows))
    }

    fn read_excel(&self, path: &PathBuf) -> Result<Value, String> {
        let mut workbook: Xlsx<_> =
            open_workbook(path).map_err(|e| format!("Cannot open Excel file: {}", e))?;

        // Read first sheet
        let sheet_name = workbook
            .sheet_names()
            .first()
            .cloned()
            .ok_or("No sheets found")?;

        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut rows = Vec::new();
            let mut headers = Vec::new();

            for (i, row) in range.rows().enumerate() {
                if i == 0 {
                    // Capture headers
                    for cell in row {
                        headers.push(cell.to_string());
                    }
                    continue;
                }

                let mut row_map = HashMap::new();
                for (j, cell) in row.iter().enumerate() {
                    if let Some(header) = headers.get(j) {
                        let val = match cell {
                            Data::Empty => "".to_string(),
                            Data::String(s) => s.to_string(),
                            Data::Float(f) => f.to_string(),
                            Data::Int(i) => i.to_string(),
                            Data::Bool(b) => b.to_string(),
                            Data::DateTime(f) => f.to_string(),
                            Data::Error(_) => "Error".to_string(),
                            Data::DateTimeIso(d) => d.to_string(),
                            Data::DurationIso(d) => d.to_string(),
                        };
                        row_map.insert(header.to_string(), val);
                    }
                }
                rows.push(row_map);
                if rows.len() >= 1000 {
                    break;
                }
            }
            Ok(json!(rows))
        } else {
            Err("Could not read sheet range".to_string())
        }
    }

    fn read_pdf(&self, path: &PathBuf) -> Result<Value, String> {
        let text = extract_text(path).map_err(|e| format!("PDF extraction failed: {}", e))?;
        // Truncate if too long?
        if text.len() > 100_000 {
            Ok(json!(text[..100_000]))
        } else {
            Ok(json!(text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_read_csv() {
        // Create CSV file
        let mut file = NamedTempFile::new().expect("failed callback");
        writeln!(file, "Name,Age\nAlice,30").expect("write failed");
        let path = file.path().to_path_buf();

        // Run tool
        let tool = OfficeTool;
        let result = tool.read_csv(&path).expect("read failed");

        let json = result.as_array().expect("expected array");
        assert_eq!(json.len(), 1);

        let obj = json[0].as_object().expect("expected object");
        assert_eq!(obj.get("Name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("Age").unwrap().as_str().unwrap(), "30");
    }

    #[tokio::test]
    async fn test_workflow_write_docx() {
        // Simulates the flow: List files (mocked data) -> Create DOCX
        let tool = OfficeTool;

        // 1. "Agent" lists files (mocked)
        let files = vec!["file1.txt", "script.py", "image.png"];
        let content = format!("Files in directory:\n{}", files.join("\n"));

        // 2. Agent creates DOCX
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("file_list.docx");

        let args = json!({
            "operation": "write_docx",
            "path": file_path.to_string_lossy(),
            "content": content
        });

        let result = tool.write_docx(&file_path, &args);
        assert!(result.is_ok());

        // 3. Verify file exists
        assert!(file_path.exists());

        // 4. (Optional) Verify content is readable back?
        // We do not have a robust read_docx method implemented, but we verified the file was created.
        println!("Created DOCX at {:?}", file_path);
    }
}
