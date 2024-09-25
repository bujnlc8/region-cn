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
