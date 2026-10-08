use core_common::{CoreError, CoreResult};
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize)]
pub struct ZipEntryInfo {
    pub name: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub is_dir: bool,
}

pub struct ZipReader { path: PathBuf }

impl ZipReader {
    pub fn open<P: AsRef<Path>>(path: P) -> CoreResult<Self> {
        let path = path.as_ref();
        if !path.exists() { return Err(CoreError::InvalidArgument(format!("file does not exist: {}", path.display()))); }
        Ok(Self { path: path.to_path_buf() })
    }

    pub fn entries(&self) -> CoreResult<Vec<ZipEntryInfo>> {
        let file = File::open(&self.path)?;
        let mut archive = ZipArchive::new(file).map_err(|e| CoreError::Archive(e.to_string()))?;
        let mut result = Vec::with_capacity(archive.len());
        for index in 0..archive.len() {
            let entry = archive.by_index(index).map_err(|e| CoreError::Archive(e.to_string()))?;
            result.push(ZipEntryInfo {
                name: entry.name().to_string(),
                compressed_size: entry.compressed_size(),
                uncompressed_size: entry.size(),
                is_dir: entry.is_dir(),
            });
        }
        Ok(result)
    }

    pub fn contains(&self, name: &str) -> CoreResult<bool> {
        let file = File::open(&self.path)?;
        let mut archive = ZipArchive::new(file).map_err(|e| CoreError::Archive(e.to_string()))?;
        Ok(archive.by_name(name).is_ok())
    }
}

pub struct ApkReader { zip: ZipReader }

impl ApkReader {
    pub fn open<P: AsRef<Path>>(path: P) -> CoreResult<Self> { Ok(Self { zip: ZipReader::open(path)? }) }
    pub fn is_apk(&self) -> CoreResult<bool> { Ok(self.zip.contains("AndroidManifest.xml")? && self.zip.contains("classes.dex")?) }
    pub fn entries(&self) -> CoreResult<Vec<ZipEntryInfo>> { self.zip.entries() }
}
