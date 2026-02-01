//! Office tool for reading/writing office files (Excel, CSV, PDF, Word)

use super::{AnyCoworkTool, ToolError};
use crate::permissions::{PermissionManager, PermissionRequest, PermissionType};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

// CSV
use csv::ReaderBuilder;

// Excel
use calamine::{open_workbook, Data, Reader, Xlsx};

// PDF
use pdf_extract::extract_text;

// Docx
use docx_rs::*;

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum OfficeArgs {
    ReadCsv {
        /// Relative path to the file
        path: String,
    },
    ReadExcel {
        /// Relative path to the file
        path: String,
    },
    ReadPdf {
        /// Relative path to the file
        path: String,
    },
    WriteDocx {
        /// Relative path to the file
        path: String,
        /// Content to write
        content: String,
    },
}

#[derive(Serialize, Deserialize)]
pub struct OfficeOutput(serde_json::Value);

/// Tool for reading/writing office files
pub struct OfficeTool {
    /// Workspace path
    pub workspace_path: PathBuf,
    /// Permission manager
    pub permissions: Arc<PermissionManager>,
}

impl OfficeTool {
    /// Create a new office tool
    pub fn new(workspace_path: PathBuf, permissions: Arc<PermissionManager>) -> Self {
        Self {
            workspace_path,
            permissions,
        }
    }

    fn validate_path(&self, path_str: &str) -> Result<PathBuf, ToolError> {
        if path_str.contains("..") || path_str.starts_with('/') {
            return Err(ToolError::validation_failed(
                "Paths must be relative and cannot contain '..'",
            ));
        }
        Ok(self.workspace_path.join(path_str))
    }

    fn read_csv(&self, path: &PathBuf) -> Result<serde_json::Value, ToolError> {
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)
            .map_err(|e| ToolError::execution_failed(format!("Failed to open CSV: {}", e)))?;

        let headers = rdr
            .headers()
            .map_err(|e| ToolError::execution_failed(e.to_string()))?
            .clone();
        let mut rows = Vec::new();

        for result in rdr.records() {
            let record = result.map_err(|e| ToolError::execution_failed(e.to_string()))?;
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

    fn read_excel(&self, path: &PathBuf) -> Result<serde_json::Value, ToolError> {
        let mut workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| ToolError::execution_failed(format!("Cannot open Excel file: {}", e)))?;

        // Read first sheet
        let sheet_name = workbook
            .sheet_names()
            .first()
            .cloned()
            .ok_or_else(|| ToolError::execution_failed("No sheets found"))?;

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
            Err(ToolError::execution_failed("Could not read sheet range"))
        }
    }

    fn read_pdf(&self, path: &PathBuf) -> Result<serde_json::Value, ToolError> {
        let text = extract_text(path)
            .map_err(|e| ToolError::execution_failed(format!("PDF extraction failed: {}", e)))?;

        // Truncate if too long
        if text.len() > 100_000 {
            Ok(json!(&text[..100_000]))
        } else {
            Ok(json!(text))
        }
    }

    fn write_docx(&self, path: &PathBuf, content: &str) -> Result<serde_json::Value, ToolError> {
        // Ensure parent dir exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(ToolError::Io)?;
        }

        let file = File::create(path).map_err(ToolError::Io)?;

        let mut doc = Docx::new();
        // Split by newlines to create paragraphs
        for line in content.split('\n') {
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(line)));
        }

        doc.build()
            .pack(file)
            .map_err(|e| ToolError::execution_failed(format!("Failed to write docx: {}", e)))?;

        Ok(json!("Document created successfully"))
    }
}

impl Tool for OfficeTool {
    const NAME: &'static str = "office";

    type Error = ToolError;
    type Args = OfficeArgs;
    type Output = OfficeOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Read content from Office files (Excel, CSV, PDF, Word). Supported formats: .xlsx, .csv, .pdf".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(OfficeArgs)).unwrap(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let (op_name, path_str) = match &args {
            OfficeArgs::ReadCsv { path } => ("read_csv", path),
            OfficeArgs::ReadExcel { path } => ("read_excel", path),
            OfficeArgs::ReadPdf { path } => ("read_pdf", path),
            OfficeArgs::WriteDocx { path, .. } => ("write_docx", path),
        };

        let target_path = self.validate_path(path_str)?;

        // Permission check
        let permission_type = if op_name == "write_docx" {
            PermissionType::FilesystemWrite
        } else {
            PermissionType::FilesystemRead
        };

        let perm_req = PermissionRequest::new(
            permission_type,
            format!(
                "Agent wants to {} office file at {}",
                op_name.replace('_', " "),
                path_str
            ),
        )
        // .with_session_id(&ctx.session_id)
        .with_resource(path_str)
        .with_metadata("operation", op_name);

        let allowed = self
            .permissions
            .check(perm_req)
            .await
            .map_err(ToolError::other)?;

        if !allowed {
            return Err(ToolError::permission_denied("User denied permission"));
        }

        // For read operations, check if file exists
        if op_name != "write_docx" && !target_path.exists() {
            return Err(ToolError::execution_failed("File not found"));
        }

        let result = match args {
            OfficeArgs::ReadCsv { .. } => self.read_csv(&target_path),
            OfficeArgs::ReadExcel { .. } => self.read_excel(&target_path),
            OfficeArgs::ReadPdf { .. } => self.read_pdf(&target_path),
            OfficeArgs::WriteDocx { content, .. } => self.write_docx(&target_path, &content),
        }?;

        Ok(OfficeOutput(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::AllowAllHandler;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_csv() {
        // Create CSV file
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        writeln!(file, "Name,Age\nAlice,30").expect("write failed");
        let path = file.path().to_path_buf();

        // Run tool
        let permissions = Arc::new(PermissionManager::new(AllowAllHandler));
        let tool = OfficeTool::new(PathBuf::from("."), permissions);
        let result = tool.read_csv(&path).expect("read failed");

        let json = result.as_array().expect("expected array");
        assert_eq!(json.len(), 1);

        let obj = json[0].as_object().expect("expected object");
        assert_eq!(obj.get("Name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(obj.get("Age").unwrap().as_str().unwrap(), "30");
    }

    #[test]
    fn test_write_docx() {
        let permissions = Arc::new(PermissionManager::new(AllowAllHandler));
        let tool = OfficeTool::new(PathBuf::from("."), permissions);
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.docx");

        let content = "Hello\nWorld";
        let result = tool.write_docx(&file_path, content);
        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn test_validate_args() {
        let permissions = Arc::new(PermissionManager::new(AllowAllHandler));
        let tool = OfficeTool::new(PathBuf::from("."), permissions);

        // Valid path
        assert!(tool.validate_path("data/test.csv").is_ok());

        // Path traversal
        assert!(tool.validate_path("../data/test.csv").is_err());
    }
}

#[async_trait::async_trait]
impl AnyCoworkTool for OfficeTool {
    fn needs_summarization(&self, args: &Self::Args, _result: &Self::Output) -> bool {
        !matches!(args, OfficeArgs::WriteDocx { .. })
    }
}
