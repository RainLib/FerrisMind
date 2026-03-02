use std::io::Write;

use calamine::{DataType, Reader, open_workbook_auto};

use super::{DocumentParser, IngestFile, ParseResult};

/// Parser for .xls (Excel 97–2003) and .xlsx (Office Open XML Workbook).
pub struct ExcelParser;

impl DocumentParser for ExcelParser {
    fn supported_mime_types(&self) -> &[&str] {
        &[
            "application/vnd.ms-excel",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "application/x-xls",
            "application/x-xlsx",
        ]
    }

    fn parse(&self, file: &IngestFile) -> anyhow::Result<ParseResult> {
        let ext = if file.filename.to_lowercase().ends_with(".xlsx") {
            ".xlsx"
        } else {
            ".xls"
        };
        let mut tmp = tempfile::Builder::new().suffix(ext).tempfile()?;
        tmp.write_all(&file.data)?;
        tmp.flush()?;
        let path = tmp.path();

        let mut workbook = open_workbook_auto(path)
            .map_err(|e| anyhow::anyhow!("Failed to open workbook '{}': {}", file.filename, e))?;

        let sheet_names = workbook.sheet_names().to_vec();
        if sheet_names.is_empty() {
            return Ok(ParseResult {
                sections: vec!["".to_string()],
                images: vec![],
            });
        }

        let mut sections = Vec::new();
        for name in &sheet_names {
            if let Ok(range) = workbook.worksheet_range(name) {
                let mut lines = Vec::new();
                for row in range.rows() {
                    let cells: Vec<String> = row
                        .iter()
                        .map(cell_to_string)
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !cells.is_empty() {
                        lines.push(cells.join("\t"));
                    }
                }
                if !lines.is_empty() {
                    sections.push(lines.join("\n"));
                }
            }
        }

        if sections.is_empty() {
            sections.push(String::new());
        }

        Ok(ParseResult {
            sections,
            images: vec![],
        })
    }
}

fn cell_to_string(cell: &impl DataType) -> String {
    cell.as_string().unwrap_or_default()
}
