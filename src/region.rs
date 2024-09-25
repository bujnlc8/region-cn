//! Region search implement

use std::{
    fs::File,
    io::{Read, Seek},
    path::PathBuf,
};

use anyhow::{anyhow, Result};

use crate::{be_u8_slice_to_i32, trie::RegionTrie, RegionItem};

#[derive(Debug)]
pub struct Region {
    file_path: PathBuf,
    version: String,
    offset_index: i32,
    region_trier: Option<RegionTrie>,
}

impl Default for RegionTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl Region {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            version: String::new(),
            offset_index: 0,
            region_trier: None,
        }
    }

    /// 获取区域类型名称
    pub fn get_type_name(&self, t: i32) -> String {
        match t {
            1 => String::from("省"),
            2 => String::from("自治区"),
            3 => String::from("市"),
            4 => String::from("区"),
            5 => String::from("县"),
            6 => String::from("自治县"),
            7 => String::from("旗"),
            8 => String::from("盟"),
            9 => String::from("州"),
            10 => String::new(),
            _ => String::new(),
        }
    }

    /// 从 region.dat读取数据记录
    fn get_record_from_data(&mut self) -> Result<Vec<RegionItem>> {
        let mut file = File::open(&self.file_path)?;
        // 跳过版本号
        file.seek(std::io::SeekFrom::Start(4))?;
        let mut index_offset: [u8; 4] = [0; 4];
        file.read_exact(&mut index_offset)?;
        self.offset_index = i32::from_be_bytes(index_offset);
        let mut record = vec![0u8; (self.offset_index - 8) as usize];
        file.read_exact(&mut record)?;
        let mut res = Vec::new();
        while !record.is_empty() {
            let region_data = be_u8_slice_to_i32(&record[..3]);
            let region = region_data >> 4;
            let region_type = region_data % region;
            let size = record[3];
            let name_bytes: Vec<u8> = record.iter().skip(4).take(size as usize).copied().collect();
            let mut name = String::from_utf8(name_bytes)?;
            let discard_year_int = name.chars().last().unwrap() as u32;
            let mut discard_year = 0;
            if discard_year_int < 256 {
                discard_year = discard_year_int + 1980;
                name = name[..name.len() - 1].to_string();
            }
            name = format!("{name}{}", self.get_type_name(region_type));
            res.push(RegionItem {
                region_code: region.to_string(),
                name,
                region_slice: Vec::new(),
                discard_year,
            });
            record = record.iter().skip((4 + size) as usize).copied().collect();
        }
        Ok(res)
    }

    /// 构建前缀树
    fn create_trier(&mut self) -> Result<RegionTrie> {
        let mut trier = RegionTrie::new();
        self.get_record_from_data()?
            .iter()
            .for_each(|x| trier.insert(x.region_code.clone(), x.name.clone(), x.discard_year));
        Ok(trier)
    }

    /// 通过前缀树来搜索结果
    pub fn search_with_trie(&mut self, region_code: &str) -> Result<RegionItem> {
        if region_code.len() != 6 {
            return Err(anyhow!("region_code's length must be 6"));
        }
        if self.region_trier.is_none() {
            let trier = self.create_trier().unwrap();
            self.region_trier = Some(trier);
        }
        self.region_trier.clone().unwrap().search(region_code)
    }

    /// 获取数据版本号
    pub fn get_version(&mut self) -> Result<&str> {
        if !self.version.is_empty() {
            return Ok(&self.version);
        }
        let mut file = File::open(&self.file_path)?;
        let mut version_bytes: [u8; 4] = [0; 4];
        file.read_exact(&mut version_bytes)?;
        self.version = i32::from_be_bytes(version_bytes).to_string();
        Ok(&self.version)
    }

    /// 从region.dat搜索数据
    pub fn search_with_data(&mut self, region_code: &str) -> Result<RegionItem> {
        if region_code.len() != 6 {
            return Err(anyhow!("region_code's length must be 6"));
        }
        let mut file = File::open(&self.file_path)?;
        file.seek(std::io::SeekFrom::Start(4))?;
        let mut index_offset: [u8; 4] = [0; 4];
        file.read_exact(&mut index_offset)?;
        self.offset_index = i32::from_be_bytes(index_offset);
        // 查找索引区
        file.seek(std::io::SeekFrom::Start(self.offset_index as u64))?;
        let mut region_code_offset: [u8; 5] = [0u8; 5];
        let region_code_int: i32 = region_code.parse()?;
        let mut offset = 0;
        for _ in 0..50 {
            let amount = file.read(&mut region_code_offset)?;
            if amount == 0 {
                break;
            }
            if region_code_offset.first().unwrap() == &((region_code_int / 10000) as u8) {
                offset = i32::from_be_bytes(region_code_offset[1..].try_into()?);
                break;
            }
        }
        if offset == 0 {
            return Err(anyhow!("cannot find record"));
        }
        file.seek(std::io::SeekFrom::Start(offset as u64))?;
        let mut province_record: [u8; 6000] = [0u8; 6000];
        let _ = file.read(&mut province_record)?;
        let search_codes = [
            format!("{}0000", &region_code[..2]),
            format!("{}00", &region_code[..4]),
            region_code.to_string(),
        ];
        let mut region_slice = Vec::new();
        let mut offset = 0;
        let mut discard_year = 0;
        while offset < 6000 {
            let region_data = be_u8_slice_to_i32(&province_record[offset..(3 + offset)]);
            let region = region_data >> 4;
            if region / 10000 != region_code_int / 10000 {
                break;
            }
            let region_type = region_data % region;
            let size = province_record[3 + offset];
            if search_codes.contains(&region.to_string()) {
                let name_bytes: Vec<u8> = province_record
                    .iter()
                    .skip(4 + offset)
                    .take(size as usize)
                    .copied()
                    .collect();
                let mut name = String::from_utf8(name_bytes)?;
                let discard_year_int = name.chars().last().unwrap() as u32;
                if discard_year_int < 256 {
                    discard_year = discard_year_int + 1980;
                    name = name[..name.len() - 1].to_string();
                }
                name = format!("{name}{}", self.get_type_name(region_type));
                region_slice.push(name);
            }
            offset += (4 + size) as usize;
        }
        if region_slice.is_empty() {
            return Err(anyhow!("cannot find record"));
        }
        Ok(RegionItem {
            region_code: region_code.to_string(),
            name: region_slice.join(""),
            region_slice,
            discard_year,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region() {
        let mut region = Region::new(PathBuf::from("data/region_full.dat"));
        assert_eq!(region.get_version().unwrap(), "2024092535");
        let result = region.search_with_data("530925").unwrap();
        assert_eq!(result.name, "云南省临沧市双江拉祜族佤族布朗族傣族自治县");
        assert_eq!(
            result.region_slice,
            vec!["云南省", "临沧市", "双江拉祜族佤族布朗族傣族自治县",]
        );
        assert_eq!(result.discard_year, 0);
        let result = region.search_with_data("110103").unwrap();
        assert_eq!(result.name, "北京市崇文区");
        assert_eq!(result.discard_year, 2010);
        let result = region.search_with_trie("530925").unwrap();
        assert_eq!(result.name, "云南省临沧市双江拉祜族佤族布朗族傣族自治县");
        assert_eq!(result.discard_year, 0);
        let result = region.search_with_trie("110103").unwrap();
        assert_eq!(result.name, "北京市崇文区");
        assert_eq!(result.discard_year, 2010);
    }
}
