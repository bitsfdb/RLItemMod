use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::engine::upk::*;
use crate::engine::pivoter::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "ID")]
    pub id: serde_json::Value,
    #[serde(rename = "Product")]
    pub product: String,
    #[serde(rename = "AssetPackage")]
    pub asset_package: String,
    #[serde(rename = "AssetPath")]
    pub asset_path: String,
    #[serde(rename = "Slot")]
    pub slot: String,
}

impl Item {
    pub fn package_stem(&self) -> String {
        self.asset_package.split('.').next().unwrap_or(&self.asset_package).to_string()
    }

    pub fn asset_parts(&self) -> Vec<String> {
        self.asset_path.split('.').map(|s| s.to_string()).collect()
    }

    pub fn asset_base(&self) -> String {
        self.asset_parts().pop().unwrap_or_default()
    }
}

pub fn infer_name_pairs(target: &Item, donor: &Item) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let target_parts = target.asset_parts();
    let donor_parts = donor.asset_parts();

    if target_parts.len() == donor_parts.len() {
        for (old, new) in donor_parts.iter().zip(target_parts.iter()) {
            pairs.push((old.clone(), new.clone()));
        }
    } else {
        if !donor_parts.is_empty() && !target_parts.is_empty() {
            pairs.push((donor_parts[0].clone(), target_parts[0].clone()));
            pairs.push((donor_parts.last().unwrap().clone(), target_parts.last().unwrap().clone()));
        }
    }
    pairs.push((donor.package_stem(), target.package_stem()));
    pairs
}

pub fn swap_asset(target: &Item, donor: &Item, game_dir: &Path) -> Result<(), String> {
    if target.asset_package.is_empty() {
        return Err(format!("Target item '{}' has no asset package in the database", target.product));
    }
    if donor.asset_package.is_empty() {
        return Err(format!("Donor item '{}' has no asset package in the database", donor.product));
    }

    let target_pkg = if target.asset_package.ends_with(".upk") {
        target.asset_package.clone()
    } else {
        format!("{}.upk", target.asset_package)
    };
    let donor_pkg = if donor.asset_package.ends_with(".upk") {
        donor.asset_package.clone()
    } else {
        format!("{}.upk", donor.asset_package)
    };

    let target_upk_path = game_dir.join(&target_pkg);
    let donor_upk_path = game_dir.join(&donor_pkg);

    if !target_upk_path.exists() { return Err(format!("Target UPK not found: {:?}", target_upk_path)); }
    if !donor_upk_path.exists() { return Err(format!("Donor UPK not found: {:?}", donor_upk_path)); }

    let bak_path = target_upk_path.with_extension("upk.bak");
    if !bak_path.exists() {
        std::fs::copy(&target_upk_path, &bak_path).map_err(|e| e.to_string())?;
    }

    let target_bytes = std::fs::read(&target_upk_path).map_err(|e| e.to_string())?;
    let mut reader = std::io::Cursor::new(&target_bytes);
    let summary = parse_file_summary(&mut reader).map_err(|e| e.to_string())?;
    if summary.compression_flags != 0 {
        return Err("Compressed packages are not yet supported in native swap (Phase 3 in progress)".to_string());
    }

    let package = ParsedPackage {
        file_path: target_upk_path.clone(),
        summary,
        names: Vec::new(),
        imports: Vec::new(),
        exports: Vec::new(),
        file_bytes: target_bytes,
    };

    let pairs = infer_name_pairs(target, donor);
    let modified = apply_name_pairs(&package, &pairs).map_err(|e| e.to_string())?;
    std::fs::write(&target_upk_path, modified).map_err(|e| e.to_string())?;

    Ok(())
}