//! The extension decides the format; the signature confirms it (spec 04).

use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::Format;

/// What the content says about a file with a document extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature {
    /// The content matches the format. An invalid document still appears,
    /// with a warning, and does not open.
    pub valid: bool,
    /// Opening needs a password (DOCX in the Office encrypted container).
    pub protected: bool,
}

/// Reads the start of `path`, following symlinks, and checks it against
/// `format`. A file that cannot be read is invalid.
pub fn check_signature(format: Format, path: &Path) -> Signature {
    match format {
        Format::Pdf => unprotected(has_pdf_header(path)),
        Format::Docx => docx_signature(path),
        Format::Md => unprotected(has_no_nul_at_start(path)),
        Format::Txt => unprotected(is_utf16(path) || has_no_nul_at_start(path)),
    }
}

fn unprotected(valid: bool) -> Signature {
    Signature {
        valid,
        protected: false,
    }
}

/// A DOCX is a ZIP, or, when password protected, the Office encrypted
/// container (OLE compound file) holding the encrypted package.
fn docx_signature(path: &Path) -> Signature {
    if is_docx_zip(path) {
        return unprotected(true);
    }
    let encrypted = is_encrypted_office(path);
    Signature {
        valid: encrypted,
        protected: encrypted,
    }
}

fn is_encrypted_office(path: &Path) -> bool {
    cfb::open(path).is_ok_and(|compound| {
        compound.is_stream("EncryptionInfo") && compound.is_stream("EncryptedPackage")
    })
}

/// PDF readers accept the header anywhere in the first 1024 bytes.
fn has_pdf_header(path: &Path) -> bool {
    read_prefix(path, 1024).is_some_and(|start| start.windows(5).any(|w| w == b"%PDF-"))
}

/// A ZIP holding `word/document.xml`. Only the central directory is read.
fn is_docx_zip(path: &Path) -> bool {
    let Ok(file) = File::open(path) else {
        return false;
    };
    zip::ZipArchive::new(file).is_ok_and(|zip| zip.index_for_name("word/document.xml").is_some())
}

/// Text files have no NUL byte in their first 8 KiB.
fn has_no_nul_at_start(path: &Path) -> bool {
    read_prefix(path, TEXT_PROBE_LEN).is_some_and(|start| !start.contains(&0))
}

/// UTF-16 text is full of NUL bytes, so its byte order mark decides.
fn is_utf16(path: &Path) -> bool {
    read_prefix(path, 2).is_some_and(|bom| bom == b"\xff\xfe" || bom == b"\xfe\xff")
}

const TEXT_PROBE_LEN: u64 = 8 * 1024;

/// Up to `len` bytes from the start of `path`; `None` if it cannot be read.
fn read_prefix(path: &Path, len: u64) -> Option<Vec<u8>> {
    let mut start = Vec::new();
    File::open(path)
        .ok()?
        .take(len)
        .read_to_end(&mut start)
        .ok()?;
    Some(start)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const VALID: Signature = Signature {
        valid: true,
        protected: false,
    };
    const INVALID: Signature = Signature {
        valid: false,
        protected: false,
    };

    fn check(format: Format, content: &[u8]) -> Signature {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("arquivo");
        fs::write(&path, content).unwrap();
        check_signature(format, &path)
    }

    #[test]
    fn pdf_with_header_in_first_1024_bytes_is_valid() {
        let content = [vec![b' '; 1000], b"%PDF-1.7\n".to_vec()].concat();

        assert_eq!(check(Format::Pdf, &content), VALID);
    }

    #[test]
    fn pdf_with_html_content_is_invalid() {
        let content = b"<!DOCTYPE html><html><body>Erro 404</body></html>";

        assert_eq!(check(Format::Pdf, content), INVALID);
    }

    #[test]
    fn pdf_header_after_1024_bytes_is_invalid() {
        let content = [vec![b' '; 1020], b"%PDF-1.7\n".to_vec()].concat();

        assert_eq!(check(Format::Pdf, &content), INVALID);
    }

    /// A ZIP with the given entry names, each with a little content.
    fn zip_with(names: &[&str]) -> Vec<u8> {
        use std::io::Write;
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for name in names {
            writer.start_file(*name, options).unwrap();
            writer.write_all(b"<xml/>").unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn docx_zip_with_word_document_xml_is_valid() {
        let content = zip_with(&["[Content_Types].xml", "word/document.xml"]);

        assert_eq!(check(Format::Docx, &content), VALID);
    }

    #[test]
    fn zip_without_word_document_xml_is_invalid_docx() {
        let content = zip_with(&["xl/workbook.xml"]);

        assert_eq!(check(Format::Docx, &content), INVALID);
    }

    /// An OLE compound file with the given stream names.
    fn cfb_with(streams: &[&str]) -> Vec<u8> {
        use std::io::Write;
        let mut compound = cfb::CompoundFile::create(std::io::Cursor::new(Vec::new())).unwrap();
        for name in streams {
            compound
                .create_stream(name)
                .unwrap()
                .write_all(b"dados")
                .unwrap();
        }
        compound.flush().unwrap();
        compound.into_inner().into_inner()
    }

    #[test]
    fn encrypted_office_container_is_valid_protected_docx() {
        let content = cfb_with(&["EncryptionInfo", "EncryptedPackage"]);

        assert_eq!(
            check(Format::Docx, &content),
            Signature {
                valid: true,
                protected: true
            }
        );
    }

    #[test]
    fn cfb_without_encryption_streams_is_invalid_docx() {
        let old_doc = cfb_with(&["WordDocument", "1Table"]);

        assert_eq!(check(Format::Docx, &old_doc), INVALID);
    }

    #[test]
    fn md_without_nul_in_first_8_kib_is_valid() {
        let content = [
            b"# Notas\n\nTexto com acentua\xc3\xa7\xc3\xa3o.\n".as_slice(),
            &[0; 4],
        ]
        .concat();
        let late_nul = [vec![b'a'; 8 * 1024], content].concat();

        assert_eq!(check(Format::Md, &late_nul), VALID);
    }

    #[test]
    fn md_with_nul_at_start_is_invalid() {
        let image_renamed = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";

        assert_eq!(check(Format::Md, image_renamed), INVALID);
    }

    #[test]
    fn txt_with_utf16_bom_is_valid_even_with_nul_bytes() {
        let little_endian = b"\xff\xfeO\x00l\x00\xe1\x00";
        let big_endian = b"\xfe\xff\x00O\x00l\x00\xe1";

        assert_eq!(check(Format::Txt, little_endian), VALID);
        assert_eq!(check(Format::Txt, big_endian), VALID);
    }

    #[test]
    fn empty_text_file_is_valid_and_empty_pdf_is_invalid() {
        assert_eq!(check(Format::Md, b""), VALID);
        assert_eq!(check(Format::Txt, b""), VALID);
        assert_eq!(check(Format::Pdf, b""), INVALID);
    }

    #[test]
    fn symlink_to_file_is_recognized_through_the_link() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("Boleto.pdf");
        fs::write(&target, b"%PDF-1.4\n").unwrap();
        let link = dir.path().join("Atalho.pdf");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert_eq!(check_signature(Format::Pdf, &link), VALID);
    }

    #[test]
    fn broken_symlink_is_invalid_document() {
        let dir = tempfile::tempdir().unwrap();
        let link = dir.path().join("Atalho.pdf");
        std::os::unix::fs::symlink(dir.path().join("sumiu.pdf"), &link).unwrap();

        assert_eq!(check_signature(Format::Pdf, &link), INVALID);
    }
}
