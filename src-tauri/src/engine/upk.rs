use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use serde::{Serialize, Deserialize};

pub const PACKAGE_FILE_TAG: u32 = 0x9E2A83C1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FNameRef {
    pub name_index: i32,
    pub instance_number: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameEntry {
    pub index: i32,
    pub name: String,
    pub flags: u64,
}

impl NameEntry {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut writer = std::io::Cursor::new(&mut buf);
        writer.write_fstring(&self.name).unwrap();
        writer.write_u64::<LittleEndian>(self.flags).unwrap();
        buf
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEntry {
    pub table_index: i32,
    pub class_package: FNameRef,
    pub class_name: FNameRef,
    pub outer_index: i32,
    pub object_name: FNameRef,
}

impl ImportEntry {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut writer = std::io::Cursor::new(&mut buf);
        writer.write_i32::<LittleEndian>(self.class_package.name_index).unwrap();
        writer.write_i32::<LittleEndian>(self.class_package.instance_number).unwrap();
        writer.write_i32::<LittleEndian>(self.class_name.name_index).unwrap();
        writer.write_i32::<LittleEndian>(self.class_name.instance_number).unwrap();
        writer.write_i32::<LittleEndian>(self.outer_index).unwrap();
        writer.write_i32::<LittleEndian>(self.object_name.name_index).unwrap();
        writer.write_i32::<LittleEndian>(self.object_name.instance_number).unwrap();
        buf
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEntry {
    pub table_index: i32,
    pub class_index: i32,
    pub super_index: i32,
    pub outer_index: i32,
    pub object_name: FNameRef,
    pub archetype_index: i32,
    pub object_flags: u64,
    pub serial_size: i32,
    pub serial_offset: i32,
    pub export_flags: u32,
    pub net_objects: Vec<i32>,
    pub package_guid: [u32; 4],
    pub package_flags: u32,
}

impl ExportEntry {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut writer = std::io::Cursor::new(&mut buf);
        writer.write_i32::<LittleEndian>(self.class_index).unwrap();
        writer.write_i32::<LittleEndian>(self.super_index).unwrap();
        writer.write_i32::<LittleEndian>(self.outer_index).unwrap();
        writer.write_i32::<LittleEndian>(self.object_name.name_index).unwrap();
        writer.write_i32::<LittleEndian>(self.object_name.instance_number).unwrap();
        writer.write_i32::<LittleEndian>(self.archetype_index).unwrap();
        writer.write_u64::<LittleEndian>(self.object_flags).unwrap();
        writer.write_i32::<LittleEndian>(self.serial_size).unwrap();
        writer.write_i32::<LittleEndian>(self.serial_offset).unwrap();
        writer.write_u32::<LittleEndian>(self.export_flags).unwrap();
        writer.write_i32::<LittleEndian>(self.net_objects.len() as i32).unwrap();
        for &id in &self.net_objects {
            writer.write_i32::<LittleEndian>(id).unwrap();
        }
        for &id in &self.package_guid {
            writer.write_u32::<LittleEndian>(id).unwrap();
        }
        writer.write_u32::<LittleEndian>(self.package_flags).unwrap();
        buf
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FCompressedChunk {
    pub uncompressed_offset: i32,
    pub uncompressed_size: i32,
    pub compressed_offset: i32,
    pub compressed_size: i32,
}

#[derive(Debug, Clone, Default)]
pub struct FileSummary {
    pub tag: u32,
    pub file_version: i16,
    pub licensee_version: i16,
    pub total_header_size: i32,
    pub folder_name: String,
    pub package_flags: u32,
    pub name_count: i32,
    pub name_offset: i32,
    pub export_count: i32,
    pub export_offset: i32,
    pub import_count: i32,
    pub import_offset: i32,
    pub depends_offset: i32,
    pub compression_flags: u32,
    pub compressed_chunks: Vec<FCompressedChunk>,
    pub guid: [u32; 4],
}

pub trait UE3ReadExt: Read + Seek {
    fn read_fstring(&mut self) -> std::io::Result<String> {
        let length = self.read_i32::<LittleEndian>()?;
        if length == 0 {
            return Ok(String::new());
        }

        if length < 0 {
            let char_count = (-length) as usize;
            let mut buf = vec![0u8; char_count * 2];
            self.read_exact(&mut buf)?;
            let u16_buf: Vec<u16> = buf
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let s = String::from_utf16_lossy(&u16_buf);
            Ok(s.trim_end_matches('\0').to_string())
        } else {
            let mut buf = vec![0u8; length as usize];
            self.read_exact(&mut buf)?;
            let s = String::from_utf8_lossy(&buf);
            Ok(s.trim_end_matches('\0').to_string())
        }
    }
}

impl<R: Read + Seek> UE3ReadExt for R {}

pub trait UE3WriteExt: Write + Seek {
    fn write_fstring(&mut self, s: &str) -> std::io::Result<()> {
        if s.is_empty() {
            self.write_i32::<LittleEndian>(0)?;
            return Ok(());
        }

        let bytes = s.as_bytes();
        let length = (bytes.len() + 1) as i32;
        self.write_i32::<LittleEndian>(length)?;
        self.write_all(bytes)?;
        self.write_u8(0)?;
        Ok(())
    }
}

impl<W: Write + Seek> UE3WriteExt for W {}

pub struct ParsedPackage {
    pub file_path: PathBuf,
    pub summary: FileSummary,
    pub names: Vec<NameEntry>,
    pub imports: Vec<ImportEntry>,
    pub exports: Vec<ExportEntry>,
    pub file_bytes: Vec<u8>,
}

impl ParsedPackage {
    pub fn object_data(&self, export: &ExportEntry) -> &[u8] {
        let start = export.serial_offset as usize;
        let end = start + export.serial_size as usize;
        if end > self.file_bytes.len() {
            return &[];
        }
        &self.file_bytes[start..end]
    }

    pub fn resolve_name(&self, ref_val: FNameRef) -> String {
        let idx = ref_val.name_index as usize;
        if idx >= self.names.len() {
            return format!("Unknown_{}", idx);
        }
        let base = &self.names[idx].name;
        if ref_val.instance_number > 0 {
            format!("{}_{}", base, ref_val.instance_number - 1)
        } else {
            base.clone()
        }
    }

    pub fn resolve_object_path(&self, index: i32) -> String {
        if index == 0 { return "None".to_string(); }
        if index > 0 {
            let exp = &self.exports[(index - 1) as usize];
            let name = self.resolve_name(exp.object_name);
            if exp.outer_index == 0 {
                name
            } else {
                format!("{}.{}", self.resolve_object_path(exp.outer_index), name)
            }
        } else {
            let imp = &self.imports[(-index - 1) as usize];
            let name = self.resolve_name(imp.object_name);
            if imp.outer_index == 0 {
                name
            } else {
                format!("{}.{}", self.resolve_object_path(imp.outer_index), name)
            }
        }
    }

    pub fn export_class_name(&self, export: &ExportEntry) -> String {
        if export.class_index == 0 {
            "Class".to_string()
        } else if export.class_index > 0 {
            self.resolve_name(self.exports[(export.class_index - 1) as usize].object_name)
        } else {
            self.resolve_name(self.imports[(-export.class_index - 1) as usize].object_name)
        }
    }
}

pub fn parse_file_summary<R: Read + Seek>(mut reader: R) -> std::io::Result<FileSummary> {
    let tag = reader.read_u32::<LittleEndian>()?;
    if tag != PACKAGE_FILE_TAG {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UPK tag"));
    }

    let file_version = reader.read_i16::<LittleEndian>()?;
    let licensee_version = reader.read_i16::<LittleEndian>()?;
    let total_header_size = reader.read_i32::<LittleEndian>()?;
    let folder_name = reader.read_fstring()?;
    let package_flags = reader.read_u32::<LittleEndian>()?;
    let name_count = reader.read_i32::<LittleEndian>()?;
    let name_offset = reader.read_i32::<LittleEndian>()?;
    let export_count = reader.read_i32::<LittleEndian>()?;
    let export_offset = reader.read_i32::<LittleEndian>()?;
    let import_count = reader.read_i32::<LittleEndian>()?;
    let import_offset = reader.read_i32::<LittleEndian>()?;
    let depends_offset = reader.read_i32::<LittleEndian>()?;
    reader.seek(SeekFrom::Current(12))?;
    reader.seek(SeekFrom::Current(4))?;
    let mut guid = [0u32; 4];
    for i in 0..4 {
        guid[i] = reader.read_u32::<LittleEndian>()?;
    }

    let generation_count = reader.read_i32::<LittleEndian>()?;
    for _ in 0..generation_count {
        reader.seek(SeekFrom::Current(12))?;
    }

    reader.seek(SeekFrom::Current(8))?;
    let compression_flags = reader.read_u32::<LittleEndian>()?;
    let compressed_chunk_count = reader.read_i32::<LittleEndian>()?;
    let mut compressed_chunks = Vec::new();
    for _ in 0..compressed_chunk_count {
        compressed_chunks.push(FCompressedChunk {
            uncompressed_offset: reader.read_i32::<LittleEndian>()?,
            uncompressed_size: reader.read_i32::<LittleEndian>()?,
            compressed_offset: reader.read_i32::<LittleEndian>()?,
            compressed_size: reader.read_i32::<LittleEndian>()?,
        });
    }

    Ok(FileSummary {
        tag,
        file_version,
        licensee_version,
        total_header_size,
        folder_name,
        package_flags,
        name_count,
        name_offset,
        export_count,
        export_offset,
        import_count,
        import_offset,
        depends_offset,
        compression_flags,
        compressed_chunks,
        guid,
    })
}

pub fn parse_name_entry<R: Read + Seek>(mut reader: R, index: i32) -> std::io::Result<NameEntry> {
    let name = reader.read_fstring()?;
    let flags = reader.read_u64::<LittleEndian>()?;
    Ok(NameEntry { index, name, flags })
}

pub fn parse_import_entry<R: Read + Seek>(mut reader: R, table_index: i32) -> std::io::Result<ImportEntry> {
    let class_package = FNameRef {
        name_index: reader.read_i32::<LittleEndian>()?,
        instance_number: reader.read_i32::<LittleEndian>()?,
    };
    let class_name = FNameRef {
        name_index: reader.read_i32::<LittleEndian>()?,
        instance_number: reader.read_i32::<LittleEndian>()?,
    };
    let outer_index = reader.read_i32::<LittleEndian>()?;
    let object_name = FNameRef {
        name_index: reader.read_i32::<LittleEndian>()?,
        instance_number: reader.read_i32::<LittleEndian>()?,
    };
    Ok(ImportEntry {
        table_index,
        class_package,
        class_name,
        outer_index,
        object_name,
    })
}

pub fn parse_export_entry<R: Read + Seek>(mut reader: R, table_index: i32) -> std::io::Result<ExportEntry> {
    let class_index = reader.read_i32::<LittleEndian>()?;
    let super_index = reader.read_i32::<LittleEndian>()?;
    let outer_index = reader.read_i32::<LittleEndian>()?;
    let object_name = FNameRef {
        name_index: reader.read_i32::<LittleEndian>()?,
        instance_number: reader.read_i32::<LittleEndian>()?,
    };
    let archetype_index = reader.read_i32::<LittleEndian>()?;
    let object_flags = reader.read_u64::<LittleEndian>()?;
    let serial_size = reader.read_i32::<LittleEndian>()?;
    let serial_offset = reader.read_i32::<LittleEndian>()?;
    let export_flags = reader.read_u32::<LittleEndian>()?;
    let net_count = reader.read_i32::<LittleEndian>()?;
    let mut net_objects = Vec::with_capacity(net_count as usize);
    for _ in 0..net_count {
        net_objects.push(reader.read_i32::<LittleEndian>()?);
    }
    let mut package_guid = [0u32; 4];
    for i in 0..4 {
        package_guid[i] = reader.read_u32::<LittleEndian>()?;
    }
    let package_flags = reader.read_u32::<LittleEndian>()?;

    Ok(ExportEntry {
        table_index,
        class_index,
        super_index,
        outer_index,
        object_name,
        archetype_index,
        object_flags,
        serial_size,
        serial_offset,
        export_flags,
        net_objects,
        package_guid,
        package_flags,
    })
}