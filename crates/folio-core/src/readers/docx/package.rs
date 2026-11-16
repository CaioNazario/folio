//! The parts of a DOCX package (a ZIP), read as XML trees.

use std::io::{Cursor, Read};

use zip::ZipArchive;
use zip::result::ZipError;

use super::xml::{self, Element};
use crate::readers::ReadError;

pub struct Package<'a> {
    zip: ZipArchive<Cursor<&'a [u8]>>,
}

impl<'a> Package<'a> {
    pub fn open(bytes: &'a [u8]) -> Result<Self, ReadError> {
        let zip = ZipArchive::new(Cursor::new(bytes)).map_err(failed)?;
        Ok(Self { zip })
    }

    /// The part `name` as an XML tree; failing when it is missing.
    pub fn xml(&mut self, name: &str) -> Result<Element, ReadError> {
        self.optional_xml(name)?
            .ok_or_else(|| ReadError::Failed(format!("missing part {name}")))
    }

    /// The part `name` as an XML tree, or `None` when it is missing.
    pub fn optional_xml(&mut self, name: &str) -> Result<Option<Element>, ReadError> {
        let Some(bytes) = self.bytes(name)? else {
            return Ok(None);
        };
        let text = String::from_utf8(bytes).map_err(failed)?;
        xml::parse(&text).map(Some)
    }

    /// The raw bytes of the part `name`, or `None` when it is missing.
    pub fn bytes(&mut self, name: &str) -> Result<Option<Vec<u8>>, ReadError> {
        let mut file = match self.zip.by_name(name) {
            Ok(file) => file,
            Err(ZipError::FileNotFound) => return Ok(None),
            Err(error) => return Err(failed(error)),
        };
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(failed)?;
        Ok(Some(bytes))
    }
}

fn failed(error: impl std::fmt::Display) -> ReadError {
    ReadError::Failed(error.to_string())
}
