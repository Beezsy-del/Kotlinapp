use lopdf::Document;
use std::fs;
use std::path::{Path, PathBuf};

/// Document Importer & Local PDF/Text Extractor
/// Extracts clean text from single and multi-page contract PDFs, text files, and images.
#[derive(Debug, Clone, Default)]
pub struct DocumentImporter;

impl DocumentImporter {
    pub fn new() -> Self {
        Self
    }

    /// Extract text from a single document path
    pub fn extract_text(&self, path: &Path) -> Result<String, String> {
        if !path.exists() {
            return Err(format!("File does not exist: {}", path.display()));
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "pdf" => self.extract_pdf(path),
            "txt" | "md" | "json" | "rtf" => self.extract_plain_text(path),
            "png" | "jpg" | "jpeg" | "tiff" | "webp" => self.extract_image_ocr(path),
            _ => {
                // Try reading as plain text first
                if let Ok(content) = fs::read_to_string(path) {
                    if !content.trim().is_empty() {
                        return Ok(content);
                    }
                }
                Err(format!("Unsupported file format: .{}", ext))
            }
        }
    }

    /// Extract and concatenate text across multiple selected documents (up to 10)
    pub fn extract_multiple_documents(&self, paths: &[PathBuf]) -> Result<String, String> {
        if paths.is_empty() {
            return Err("No documents provided for extraction.".into());
        }

        let mut aggregated = String::new();
        let limited_paths = &paths[..paths.len().min(10)];

        for (index, path) in limited_paths.iter().enumerate() {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown Document");

            match self.extract_text(path) {
                Ok(content) => {
                    let clean = content.trim();
                    if !clean.is_empty() {
                        aggregated.push_str(&format!("--- [Document {}: {}] ---\n", index + 1, file_name));
                        aggregated.push_str(clean);
                        aggregated.push_str("\n\n");
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Skipping document {} due to error: {}", file_name, e);
                }
            }
        }

        let final_text = aggregated.trim().to_string();
        if final_text.is_empty() {
            Err("Failed to extract readable text from the selected documents.".into())
        } else {
            Ok(final_text)
        }
    }

    /// Extract text content from all pages of a PDF file using lopdf
    pub fn extract_pdf(&self, path: &Path) -> Result<String, String> {
        let doc = Document::load(path)
            .map_err(|e| format!("Failed to parse PDF {}: {}", path.display(), e))?;

        let mut full_text = String::new();
        let pages = doc.get_pages();

        if pages.is_empty() {
            return Err("PDF contains no readable pages.".into());
        }

        for (page_num, _) in pages.iter() {
            if let Ok(text) = doc.extract_text(&[*page_num]) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    full_text.push_str(trimmed);
                    full_text.push('\n');
                }
            }
        }

        let clean = full_text.trim();
        if clean.is_empty() {
            // Some PDFs contain scanned image pages without text streams.
            // On macOS, we can attempt Apple Vision OCR on the PDF or image.
            #[cfg(target_os = "macos")]
            {
                if let Ok(ocr_text) = self.extract_macos_vision_ocr(path) {
                    if !ocr_text.trim().is_empty() {
                        return Ok(ocr_text);
                    }
                }
            }
            Err("PDF appears to contain no extractable digital text stream.".into())
        } else {
            Ok(clean.to_string())
        }
    }

    /// Read plain text files (UTF-8)
    fn extract_plain_text(&self, path: &Path) -> Result<String, String> {
        fs::read_to_string(path)
            .map_err(|e| format!("Failed to read text file {}: {}", path.display(), e))
    }

    /// Extract text from scanned physical contracts or images
    pub fn extract_image_ocr(&self, path: &Path) -> Result<String, String> {
        #[cfg(target_os = "macos")]
        {
            self.extract_macos_vision_ocr(path)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(format!(
                "Image OCR for {} requires tesseract or native OCR backend on this operating system.",
                path.display()
            ))
        }
    }

    /// Native macOS Apple Vision OCR via swift/osascript bridge
    #[cfg(target_os = "macos")]
    fn extract_macos_vision_ocr(&self, path: &Path) -> Result<String, String> {
        let abs_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        // Use swift one-liner with Apple Vision framework (VNRecognizeTextRequest)
        let script = format!(
            r#"
            import Cocoa
            import Vision

            let path = "{}"
            let url = URL(file_prefix: path)
            guard let image = NSImage(contentsOf: url),
                  let tiffData = image.tiffRepresentation,
                  let bitmap = NSBitmapImageRep(data: tiffData),
                  let cgImage = bitmap.cgImage else {{
                exit(1)
            }}

            let request = VNRecognizeTextRequest {{ req, err in
                guard let observations = req.results as? [VNRecognizedTextObservation] else {{ return }}
                for obs in observations {{
                    if let candidate = obs.topCandidates(1).first {{
                        print(candidate.string)
                    }}
                }}
            }}
            request.recognitionLevel = .accurate
            request.usesLanguageCorrection = true

            let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
            try? handler.perform([request])
            "#,
            abs_path.display()
        );

        let output = std::process::Command::new("swift")
            .arg("-e")
            .arg(&script)
            .output();

        if let Ok(res) = output {
            if res.status.success() {
                let text = String::from_utf8_lossy(&res.stdout).trim().to_string();
                if !text.is_empty() {
                    return Ok(text);
                }
            }
        }

        Err(format!("Apple Vision OCR could not extract text from {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_extract_plain_text() {
        let importer = DocumentImporter::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("verdict_edge_test_contract.txt");

        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"AGREEMENT\nClause 1: Mutual non-disclosure obligations.").unwrap();

        let extracted = importer.extract_text(&test_file).unwrap();
        assert!(extracted.contains("AGREEMENT"));
        assert!(extracted.contains("Clause 1: Mutual non-disclosure"));

        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_extract_multiple_documents() {
        let importer = DocumentImporter::new();
        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("contract_part1.txt");
        let file2 = temp_dir.join("contract_part2.txt");

        fs::write(&file1, "PART 1: Section 27 Restraint of Trade non-compete.").unwrap();
        fs::write(&file2, "PART 2: Unlimited indemnification provisions.").unwrap();

        let aggregated = importer.extract_multiple_documents(&[file1.clone(), file2.clone()]).unwrap();
        assert!(aggregated.contains("--- [Document 1: contract_part1.txt] ---"));
        assert!(aggregated.contains("PART 1: Section 27"));
        assert!(aggregated.contains("--- [Document 2: contract_part2.txt] ---"));
        assert!(aggregated.contains("PART 2: Unlimited indemnification"));

        let _ = fs::remove_file(file1);
        let _ = fs::remove_file(file2);
    }

    #[test]
    fn test_missing_file_handling() {
        let importer = DocumentImporter::new();
        let result = importer.extract_text(Path::new("/non/existent/path/contract.pdf"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File does not exist"));
    }
}
