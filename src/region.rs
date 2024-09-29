//! Region search implement

use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{Read, Seek},
    path::PathBuf,
    rc::Rc,
};

use encoding::{all::GBK, Encoding};

use crate::{be_u8_slice_to_i32, decode_u8_list, trie::RegionTrie, RegionError, RegionItem};

#[derive(Debug)]
pub struct Region {
    file_path: PathBuf,
    version: String,
    offset_index: i32,
    region_trier: Option<Rc<RefCell<RegionTrie>>>,
    char_map: Rc<RefCell<HashMap<usize, char>>>,
    file: Rc<RefCell<File>>,
}

impl Default for RegionTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl Region {
    pub fn new(file_path: PathBuf) -> Self {
        let file = File::open(&file_path).unwrap();
        Self {
            file_path,
            version: String::new(),
            offset_index: 0,
            region_trier: None,
            char_map: Rc::new(RefCell::new(HashMap::new())),
            file: Rc::new(RefCell::new(file)),
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
            10 => String::from("自治州"),
            11 => String::from("藏族自治州"),
            12 => String::from("满族自治县"),
            13 => String::from("蒙古族自治县"),
            14 => String::from("苗族自治县"),
            15 => String::from("土家族自治县"),
            _ => String::new(),
        }
    }

    /// 读取字符集
    pub fn get_char_map(&self, file_ref: &mut File) -> Result<(), RegionError> {
        let mut char_map_ref = self.char_map.borrow_mut();
        if char_map_ref.is_empty() {
            // 读取字符集
            file_ref
                .seek(std::io::SeekFrom::Start(
                    (self.offset_index + 34 * 3) as u64,
                ))
                .map_err(RegionError::IOError)?;
            // gbk编码
            let mut char_bytes = Vec::new();
            file_ref
                .read_to_end(&mut char_bytes)
                .map_err(RegionError::IOError)?;
            let chars = GBK
                .decode(&char_bytes, encoding::DecoderTrap::Strict)
                .map_err(|x| RegionError::Message(x.to_string()))?;
            let mut char_map = HashMap::new();
            for (i, c) in chars.chars().enumerate() {
                char_map.insert(i + 64, c);
            }
            *char_map_ref = char_map;
        }
        Ok(())
    }

    /// 从 region.dat读取数据记录
    pub fn get_record_from_data(&mut self) -> Result<Vec<RegionItem>, RegionError> {
        let mut file = self.file.borrow_mut();
        // 跳过版本号
        if self.offset_index == 0 {
            file.seek(std::io::SeekFrom::Start(4))
                .map_err(RegionError::IOError)?;
            let mut index_offset: [u8; 2] = [0; 2];
            file.read_exact(&mut index_offset)
                .map_err(RegionError::IOError)?;
            self.offset_index = be_u8_slice_to_i32(&index_offset);
        } else {
            file.seek(std::io::SeekFrom::Start(6))
                .map_err(RegionError::IOError)?;
        }
        let mut record = vec![0u8; (self.offset_index - 6) as usize];
        file.read_exact(&mut record).map_err(RegionError::IOError)?;
        self.get_char_map(&mut file)?;
        let char_map = self.char_map.borrow();
        let mut res = Vec::new();
        while !record.is_empty() {
            let size = be_u8_slice_to_i32(&record[..1]);
            let region_code_type = be_u8_slice_to_i32(&record[1..4]);
            let region = region_code_type >> 4;
            let region_type = region_code_type % region;
            let record_bytes: Vec<u8> = record
                .iter()
                .skip(4)
                .take((size - 4) as usize)
                .copied()
                .collect();
            let (name_char_index_list, discard_year_int) = decode_u8_list(&record_bytes);
            let mut name_chars = Vec::new();
            for i in name_char_index_list {
                name_chars.push(char_map.get(&(i as usize)).unwrap());
            }
            let mut name = String::from_iter(name_chars);
            let mut discard_year = 0;
            if discard_year_int > 0 {
                discard_year = discard_year_int + 1980;
            }
            name = format!("{name}{}", self.get_type_name(region_type));
            res.push(RegionItem {
                region_code: region.to_string(),
                name,
                region_slice: Vec::new(),
                discard_year,
            });
            record = record.iter().skip(size as usize).copied().collect();
        }
        Ok(res)
    }

    /// 构建前缀树
    fn create_trier(&mut self) -> Result<RegionTrie, RegionError> {
        let mut trier = RegionTrie::new();
        self.get_record_from_data()?
            .iter()
            .for_each(|x| trier.insert(x.region_code.clone(), x.name.clone(), x.discard_year));
        Ok(trier)
    }

    /// 通过前缀树来搜索结果
    pub fn search_with_trie(&mut self, region_code: &str) -> Result<RegionItem, RegionError> {
        if region_code.len() != 6 {
            return Err(RegionError::Message(
                "region_code's length must be 6".to_string(),
            ));
        }
        if self.region_trier.is_none() {
            let trier = self.create_trier().unwrap();
            self.region_trier = Some(Rc::new(RefCell::new(trier)));
        }
        self.region_trier
            .clone()
            .unwrap()
            .borrow()
            .search(region_code)
    }

    /// 获取数据版本号
    pub fn get_version(&mut self) -> Result<&str, RegionError> {
        if !self.version.is_empty() {
            return Ok(&self.version);
        }
        let mut file = File::open(&self.file_path).map_err(RegionError::IOError)?;
        let mut version_bytes: [u8; 4] = [0; 4];
        file.read_exact(&mut version_bytes)
            .map_err(RegionError::IOError)?;
        self.version = i32::from_be_bytes(version_bytes).to_string();
        Ok(&self.version)
    }

    /// 从region.dat搜索数据
    pub fn search_with_data(&mut self, region_code: &str) -> Result<RegionItem, RegionError> {
        let mut file = self.file.borrow_mut();
        if region_code.len() != 6 {
            return Err(RegionError::Message(
                "region_code's length must be 6".to_string(),
            ));
        }
        if self.offset_index == 0 {
            file.seek(std::io::SeekFrom::Start(4))
                .map_err(RegionError::IOError)?;
            let mut index_offset: [u8; 2] = [0; 2];
            file.read_exact(&mut index_offset)
                .map_err(RegionError::IOError)?;
            self.offset_index = be_u8_slice_to_i32(&index_offset);
        }
        // 读取字符集
        self.get_char_map(&mut file)?;
        // 查找索引区
        file.seek(std::io::SeekFrom::Start(self.offset_index as u64))
            .map_err(RegionError::IOError)?;
        let mut region_code_offset: [u8; 3] = [0u8; 3];
        let region_code_int: i32 = region_code.parse().map_err(RegionError::ParseError)?;
        let mut offset = 0;
        for _ in 0..34 {
            let amount = file
                .read(&mut region_code_offset)
                .map_err(RegionError::IOError)?;
            if amount == 0 {
                break;
            }
            // 高7位表示code的前2位，后17位表示偏移
            let combine = be_u8_slice_to_i32(&region_code_offset);
            let code_2 = combine >> 17;
            if code_2 == region_code_int / 10000 {
                offset = combine - (code_2 << 17);
                break;
            }
        }
        if offset == 0 {
            return Err(RegionError::Message("cannot find record".to_string()));
        }
        file.seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(RegionError::IOError)?;
        let mut province_record: [u8; 4000] = [0u8; 4000];
        let _ = file
            .read(&mut province_record)
            .map_err(RegionError::IOError)?;
        let search_codes = [
            format!("{}0000", &region_code[..2]),
            format!("{}00", &region_code[..4]),
            region_code.to_string(),
        ];
        let mut region_slice = Vec::new();
        let mut offset = 0;
        let mut discard_year = 0;
        let char_map = self.char_map.borrow();
        while offset < 4000 {
            let size = be_u8_slice_to_i32(&province_record[offset..1 + offset]);
            let region_code_type = be_u8_slice_to_i32(&province_record[1 + offset..4 + offset]);
            let region = region_code_type >> 4;
            // 已经到别的省份
            if region / 10000 != region_code_int / 10000 {
                break;
            }
            let region_type = region_code_type % region;
            if search_codes.contains(&region.to_string()) {
                let record_bytes: Vec<u8> = province_record
                    .iter()
                    .skip(4 + offset)
                    .take((size - 4) as usize)
                    .copied()
                    .collect();
                let (name_char_index_list, discard_year_int) = decode_u8_list(&record_bytes);
                let mut name_chars = Vec::new();
                for i in name_char_index_list {
                    name_chars.push(char_map.get(&(i as usize)).unwrap());
                }
                let mut name = String::from_iter(name_chars);
                if discard_year_int > 0 && region.to_string() == region_code {
                    discard_year = discard_year_int + 1980;
                }
                name = format!("{name}{}", self.get_type_name(region_type));
                region_slice.push(name);
            }
            offset += size as usize;
        }
        if region_slice.is_empty() {
            return Err(RegionError::Message("cannot find record".to_string()));
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
        assert_eq!(region.get_version().unwrap(), "2024092911");
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
