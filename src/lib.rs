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

#[derive(Debug)]
pub struct RegionItem {
    pub region_code: String,
    pub name: String,
    pub region_slice: Vec<String>,
}
