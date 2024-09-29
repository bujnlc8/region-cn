/*!
根据 中国 6 位行政区划代码查询地区名称

基本用法:

```
use std::path::PathBuf;

use region_cn::region::Region;

let mut region = Region::new(PathBuf::from("data/region.dat"));
// 直接在region.dat中搜索
match region.search_with_data("530925") {
    Ok(data) => {
        println!("{:#?}", data);
        assert_eq!(data.name, "云南省临沧市双江拉祜族佤族布朗族傣族自治县");
        assert_eq!(
            data.region_slice,
            vec!["云南省", "临沧市", "双江拉祜族佤族布朗族傣族自治县",]
        );
    }
    Err(e) => eprintln!("{}", e),
}
// 通过前缀树来搜索结果
let result = region.search_with_trie("530925").unwrap();
assert_eq!(result.name, "云南省临沧市双江拉祜族佤族布朗族傣族自治县");
```
*/

pub mod region;
pub mod trie;

use std::{fmt, num::ParseIntError};

/// RegionItem
#[derive(Debug)]
pub struct RegionItem {
    /// 地区代码
    pub region_code: String,
    /// 地区全称
    pub name: String,
    /// 一级 二级 三级 地区名称
    pub region_slice: Vec<String>,
    /// 废止的年份，为0表示未废止
    pub discard_year: u32,
}

/// 大端字节序列转成i32
pub(crate) fn be_u8_slice_to_i32(bytes: &[u8]) -> i32 {
    let mut res = 0;
    let length = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        res += (*b as i32) << ((length - i - 1) * 8)
    }
    res
}

/// 将vec[u8]解析成12位的数组
pub(crate) fn decode_u8_list(u8_list: &Vec<u8>) -> (Vec<u32>, u32) {
    // 按4bit分割， 再3个组合成12位
    let mut four_bits: Vec<u8> = Vec::new();
    for u8_val in u8_list {
        four_bits.push((u8_val & 0xF0) >> 4); // 高4位
        four_bits.push(u8_val & 0x0F); // 低4位
    }

    let mut res: Vec<u32> = Vec::new();
    for i in 0..four_bits.len() / 3 {
        let a = four_bits[3 * i] as u32;
        let b = four_bits[3 * i + 1] as u32;
        let c = four_bits[3 * i + 2] as u32;
        let num = (a << 8) + (b << 4) + c;
        if num >= 64 {
            res.push(num);
        }
    }

    let mut discard_year_int: u32 = 0;
    let last_four_bits = &four_bits[res.len() * 3..];
    for (i, &b) in last_four_bits.iter().enumerate() {
        discard_year_int += (b as u32) << (4 * (last_four_bits.len() - i - 1));
    }

    (res, discard_year_int)
}

/// Wrapper for Error
#[derive(Debug)]
pub enum RegionError {
    /// IOError
    IOError(std::io::Error),
    /// ParseIntError
    ParseError(ParseIntError),
    /// Message
    Message(String),
}

impl fmt::Display for RegionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RegionError::IOError(err) => write!(f, "IOError: {}", err),
            RegionError::ParseError(err) => write!(f, "ParseError: {}", err),
            RegionError::Message(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for RegionError {}
