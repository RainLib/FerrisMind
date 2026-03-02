//! Parser for legacy Microsoft Office formats (.doc, .ppt) via external tools.
//!
//! - **.doc** (application/msword): Tries `antiword -t` first (stdout), then
//!   `soffice --headless --convert-to txt` (LibreOffice).
//! - **.ppt** (application/vnd.ms-powerpoint): Uses `soffice --headless --convert-to txt`.
//!
//! Requires one of: `antiword` (for .doc only), or `soffice` (LibreOffice) for both.

use std::io::{Read, Write};

use super::{DocumentParser, IngestFile, ParseResult};

pub struct LegacyOfficeParser;

impl DocumentParser for LegacyOfficeParser {
    fn supported_mime_types(&self) -> &[&str] {
        &["application/msword", "application/vnd.ms-powerpoint"]
    }

    fn parse(&self, file: &IngestFile) -> anyhow::Result<ParseResult> {
        let is_doc = file.mime_type == "application/msword"
            || file.filename.to_lowercase().ends_with(".doc");
        let is_ppt = file.mime_type == "application/vnd.ms-powerpoint"
            || file.filename.to_lowercase().ends_with(".ppt");

        let text = if is_doc {
            try_antiword(file).or_else(|_| try_soffice_convert(file))?
        } else if is_ppt {
            try_soffice_convert(file)?
        } else {
            anyhow::bail!("Unsupported legacy format for '{}'", file.filename)
        };

        let sections = if text.trim().is_empty() {
            vec!["".to_string()]
        } else {
            text.split("\n\n")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        Ok(ParseResult {
            sections,
            images: vec![],
        })
    }
}

/// Try antiword (outputs to stdout). Only for .doc.
fn try_antiword(file: &IngestFile) -> anyhow::Result<String> {
    let mut tmp = tempfile::Builder::new().suffix(".doc").tempfile()?;
    std::io::Write::write_all(&mut tmp, &file.data)?;
    tmp.flush()?;
    let path = tmp.path();

    let out = std::process::Command::new("antiword")
        .arg("-t")
        .arg(path)
        .output()
        .map_err(|e| anyhow::anyhow!("antiword not available: {}", e))?;

    if !out.status.success() {
        anyhow::bail!("antiword failed: {}", String::from_utf8_lossy(&out.stderr));
    }

    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    if text.trim().is_empty() {
        anyhow::bail!("antiword produced no text");
    }
    Ok(text)
}

/// Use LibreOffice headless to convert to .txt, then read the file.
fn try_soffice_convert(file: &IngestFile) -> anyhow::Result<String> {
    let ext = if file.filename.to_lowercase().ends_with(".ppt") {
        ".ppt"
    } else {
        ".doc"
    };
    let tmp_dir = tempfile::tempdir()?;
    let input_name = "input".to_string() + ext;
    let input_path = tmp_dir.path().join(&input_name);
    let output_txt = tmp_dir.path().join("input.txt");

    std::fs::write(&input_path, &file.data)?;

    let soffice_bin = find_soffice();

    let status = std::process::Command::new(&soffice_bin)
        .args([
            "--headless",
            "--convert-to",
            "txt",
            "--outdir",
            tmp_dir.path().to_str().unwrap_or("."),
            input_path.to_str().unwrap_or("input.doc"),
        ])
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "soffice (LibreOffice) not available at '{}': {}",
                soffice_bin,
                e
            )
        })?
        .status;

    if !status.success() {
        anyhow::bail!(
            "soffice conversion failed. Install LibreOffice (e.g. apt install libreoffice-writer) for .doc/.ppt support."
        );
    }

    if !output_txt.exists() {
        anyhow::bail!("soffice did not produce input.txt");
    }

    let mut f = std::fs::File::open(&output_txt)
        .map_err(|e| anyhow::anyhow!("Cannot read converted file: {}", e))?;
    let mut text = String::new();
    f.read_to_string(&mut text)?;
    Ok(text)
}

/// Resolve the absolute path to `soffice`. This is necessary because IDE debuggers
/// and some process launchers may not inherit the user's full PATH.
fn find_soffice() -> String {
    let candidates = [
        "/usr/local/bin/soffice",
        "/opt/homebrew/bin/soffice",
        "/Applications/LibreOffice.app/Contents/MacOS/soffice",
        "/usr/bin/soffice",
        "/snap/bin/soffice",
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }
    // fallback: rely on PATH
    "soffice".to_string()
}
