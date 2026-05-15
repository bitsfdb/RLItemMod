use std::collections::HashMap;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use crate::engine::upk::*;

pub struct SummaryOffsets {
    pub total_header_size_offset: usize,
    pub package_flags_offset: usize,
    pub name_count_offset: usize,
    pub name_offset_offset: usize,
    pub export_count_offset: usize,
    pub export_offset_offset: usize,
    pub import_count_offset: usize,
    pub import_offset_offset: usize,
    pub depends_offset_offset: usize,
    pub import_export_guids_offset_offset: usize,
    pub thumbnail_table_offset_offset: usize,
    pub generations_count_offset: usize,
    pub generation_entries_offset: usize,
    pub generation_count: i32,
}

pub fn ensure_name_entry(names: &mut Vec<NameEntry>, text: &str) -> i32 {
    if let Some(pos) = names.iter().position(|n| n.name == text) {
        return pos as i32;
    }
    let idx = names.len() as i32;
    names.push(NameEntry { index: idx, name: text.to_string(), flags: 0 });
    idx
}

pub fn rename_name_entry(package: &ParsedPackage, index: i32, new_text: &str) -> std::io::Result<Vec<u8>> {
    let mut names = package.names.clone();
    if (index as usize) < names.len() {
        names[index as usize].name = new_text.to_string();
    }
    replace_header_tables(package, &names, &package.imports)
}

pub fn apply_name_pairs(package: &ParsedPackage, pairs: &[(String, String)]) -> std::io::Result<Vec<u8>> {
    let mut names = package.names.clone();
    let mut changed = false;
    for (old, new) in pairs {
        for entry in &mut names {
            if entry.name.eq_ignore_ascii_case(old) {
                entry.name = new.clone();
                changed = true;
            }
        }
    }
    if !changed {
        return Ok(package.file_bytes.clone());
    }
    replace_header_tables(package, &names, &package.imports)
}

pub fn find_summary_offsets(data: &[u8]) -> std::io::Result<SummaryOffsets> {
    let mut reader = std::io::Cursor::new(data);
    if reader.read_u32::<LittleEndian>()? != PACKAGE_FILE_TAG {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UPK tag"));
    }
    reader.read_u16::<LittleEndian>()?;
    reader.read_u16::<LittleEndian>()?;
    let total_header_size_offset = reader.position() as usize;
    reader.read_i32::<LittleEndian>()?;
    reader.read_fstring()?;
    let package_flags_offset = reader.position() as usize;
    reader.read_u32::<LittleEndian>()?;
    let name_count_offset = reader.position() as usize;
    reader.read_i32::<LittleEndian>()?;
    let name_offset_offset = reader.position() as usize;
    reader.read_i32::<LittleEndian>()?;
    let export_count_offset = reader.position() as usize;
    reader.read_i32::<LittleEndian>()?;
    let export_offset_offset = reader.position() as usize;
    reader.read_i32::<LittleEndian>()?;
    let import_count_offset = reader.position() as usize;
    reader.read_i32::<LittleEndian>()?;
    let import_offset_offset = reader.position() as usize;
    reader.read_i32::<LittleEndian>()?;
    let depends_offset_offset = reader.position() as usize;
    reader.read_i32::<LittleEndian>()?;
    let import_export_guids_offset_offset = reader.position() as usize;
    reader.read_i32::<LittleEndian>()?;
    reader.read_i32::<LittleEndian>()?;
    reader.read_i32::<LittleEndian>()?;
    let thumbnail_table_offset_offset = reader.position() as usize;
    reader.read_i32::<LittleEndian>()?;
    for _ in 0..4 { reader.read_u32::<LittleEndian>()?; }
    let generations_count_offset = reader.position() as usize;
    let generation_count = reader.read_i32::<LittleEndian>()?;
    let generation_entries_offset = reader.position() as usize;

    Ok(SummaryOffsets {
        total_header_size_offset,
        package_flags_offset,
        name_count_offset,
        name_offset_offset,
        export_count_offset,
        export_offset_offset,
        import_count_offset,
        import_offset_offset,
        depends_offset_offset,
        import_export_guids_offset_offset,
        thumbnail_table_offset_offset,
        generations_count_offset,
        generation_entries_offset,
        generation_count,
    })
}

pub fn patch_i32(data: &mut [u8], offset: usize, value: i32) {
    let mut writer = std::io::Cursor::new(&mut data[offset..offset + 4]);
    writer.write_i32::<LittleEndian>(value).unwrap();
}

pub fn replace_header_tables(package: &ParsedPackage, names: &[NameEntry], imports: &[ImportEntry]) -> std::io::Result<Vec<u8>> {
    let offsets = find_summary_offsets(&package.file_bytes)?;
    let old_depends_offset = package.summary.depends_offset as usize;

    let mut new_data = Vec::new();
    new_data.extend_from_slice(&package.file_bytes[..package.summary.name_offset as usize]);

    let names_blob: Vec<u8> = names.iter().flat_map(|n| n.serialize()).collect();
    let imports_blob: Vec<u8> = imports.iter().flat_map(|i| i.serialize()).collect();

    let export_offset = (package.summary.name_offset as usize) + names_blob.len() + imports_blob.len();
    let mut patched_exports = package.exports.clone();
    let export_size: usize = patched_exports.iter().map(|e| e.serialize().len()).sum();
    let depends_offset = export_offset + export_size;
    let delta = (depends_offset as i32) - (old_depends_offset as i32);

    if delta != 0 {
        for exp in &mut patched_exports {
            if exp.serial_offset >= old_depends_offset as i32 {
                exp.serial_offset += delta;
            }
        }
    }

    let exports_blob: Vec<u8> = patched_exports.iter().flat_map(|e| e.serialize()).collect();
    new_data.extend_from_slice(&names_blob);
    new_data.extend_from_slice(&imports_blob);
    new_data.extend_from_slice(&exports_blob);

    patch_i32(&mut new_data, offsets.name_count_offset, names.len() as i32);
    patch_i32(&mut new_data, offsets.name_offset_offset, package.summary.name_offset);
    patch_i32(&mut new_data, offsets.export_count_offset, patched_exports.len() as i32);
    patch_i32(&mut new_data, offsets.export_offset_offset, export_offset as i32);
    patch_i32(&mut new_data, offsets.import_count_offset, imports.len() as i32);
    patch_i32(&mut new_data, offsets.import_offset_offset, (package.summary.name_offset as usize + names_blob.len()) as i32);
    patch_i32(&mut new_data, offsets.depends_offset_offset, depends_offset as i32);

    let import_export_guids_offset_val = {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&package.file_bytes[offsets.import_export_guids_offset_offset..offsets.import_export_guids_offset_offset + 4]);
        i32::from_le_bytes(buf)
    };
    if import_export_guids_offset_val >= old_depends_offset as i32 && import_export_guids_offset_val != 0 {
        patch_i32(&mut new_data, offsets.import_export_guids_offset_offset, import_export_guids_offset_val + delta);
    }

    new_data.extend_from_slice(&package.file_bytes[old_depends_offset..]);

    Ok(new_data)
}

pub fn merge_donor_exports_as_imports(target: &ParsedPackage, donor: &ParsedPackage, donor_package_name: &str) -> std::io::Result<Vec<u8>> {
    let mut names = target.names.clone();
    let mut imports = target.imports.clone();
    let mut existing_paths: HashMap<String, i32> = HashMap::new();

    for imp in &target.imports {
        let path = target.resolve_object_path(imp.table_index);
        existing_paths.insert(path, imp.table_index);
    }

    let mut donor_cache: HashMap<i32, i32> = HashMap::new();

    let root_idx = {
        if let Some(&idx) = existing_paths.get(donor_package_name) {
            idx
        } else {
            let cp = ensure_name_entry(&mut names, "Core");
            let cn = ensure_name_entry(&mut names, "Package");
            let on = ensure_name_entry(&mut names, donor_package_name);
            let idx = -(imports.len() as i32 + 1);
            imports.push(ImportEntry {
                table_index: imports.len() as i32,
                class_package: FNameRef { name_index: cp, instance_number: 0 },
                class_name: FNameRef { name_index: cn, instance_number: 0 },
                outer_index: 0,
                object_name: FNameRef { name_index: on, instance_number: 0 },
            });
            existing_paths.insert(donor_package_name.to_string(), idx);
            idx
        }
    };

    fn ensure_donor_object(
        idx: i32,
        donor: &ParsedPackage,
        names: &mut Vec<NameEntry>,
        imports: &mut Vec<ImportEntry>,
        donor_cache: &mut HashMap<i32, i32>,
        existing_paths: &mut HashMap<String, i32>,
        donor_package_name: &str,
        root_idx: i32,
    ) -> i32 {
        if idx == 0 { return 0; }
        if let Some(&cached) = donor_cache.get(&idx) { return cached; }

        let path = donor.resolve_object_path(idx);
        let scoped_path = if idx > 0 { format!("{}.{}", donor_package_name, path) } else { path.clone() };

        if let Some(&existing) = existing_paths.get(&scoped_path) {
            donor_cache.insert(idx, existing);
            return existing;
        }

        let (obj_name_str, outer_idx_orig, class_pkg_str, class_name_str) = if idx > 0 {
            let exp = &donor.exports[(idx - 1) as usize];
            let name = donor.resolve_name(exp.object_name);
            let class_pkg = "Core".to_string();
            let class_name = donor.export_class_name(exp);
            (name, exp.outer_index, class_pkg, class_name)
        } else {
            let imp = &donor.imports[(-idx - 1) as usize];
            let name = donor.resolve_name(imp.object_name);
            let class_pkg = donor.resolve_name(imp.class_package);
            let class_name = donor.resolve_name(imp.class_name);
            (name, imp.outer_index, class_pkg, class_name)
        };

        let outer_idx = if outer_idx_orig == 0 && idx > 0 {
            root_idx
        } else {
            ensure_donor_object(outer_idx_orig, donor, names, imports, donor_cache, existing_paths, donor_package_name, root_idx)
        };

        let cp = ensure_name_entry(names, &class_pkg_str);
        let cn = ensure_name_entry(names, &class_name_str);
        let on = ensure_name_entry(names, &obj_name_str);

        let new_idx = -(imports.len() as i32 + 1);
        imports.push(ImportEntry {
            table_index: imports.len() as i32,
            class_package: FNameRef { name_index: cp, instance_number: 0 },
            class_name: FNameRef { name_index: cn, instance_number: 0 },
            outer_index: outer_idx,
            object_name: FNameRef { name_index: on, instance_number: 0 },
        });

        donor_cache.insert(idx, new_idx);
        existing_paths.insert(scoped_path, new_idx);
        new_idx
    }

    for i in 1..=donor.exports.len() as i32 {
        ensure_donor_object(i, donor, &mut names, &mut imports, &mut donor_cache, &mut existing_paths, donor_package_name, root_idx);
    }

    replace_header_tables(target, &names, &imports)
}