use std::{fs::File, io::Read, path::PathBuf};

use region_cn::region::Region;
use serde_json::Value;

pub fn main() {
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
    // 创建前缀树来搜索结果
    let result = region.search_with_trie("530925").unwrap();
    assert_eq!(result.name, "云南省临沧市双江拉祜族佤族布朗族傣族自治县");
    // 测试所有的数据
    let mut origin_file = File::open(PathBuf::from("data/region_full.txt")).unwrap();
    let mut file_string = String::new();
    origin_file.read_to_string(&mut file_string).unwrap();
    let mut searcher = Region::new(PathBuf::from("data/region_full.dat"));
    let json_data: Value = serde_json::from_str(&file_string).unwrap();
    for x in json_data.as_array().unwrap() {
        let code = x.get(0).unwrap().as_str().unwrap();
        let region_name = x.get(1).unwrap().as_str().unwrap().replace("*", "");
        let discard_year = x.get(2).unwrap().as_str().unwrap();
        let result = searcher.search_with_trie(code).unwrap();
        println!("{code} {region_name} => {}", result.name);
        assert!(result.name.ends_with(&region_name));
        if discard_year.is_empty() {
            assert_eq!(result.discard_year, 0);
        } else {
            assert_eq!(result.discard_year, discard_year.parse::<u32>().unwrap());
        }
        let result = searcher.search_with_data(code).unwrap();
        println!("{code} {region_name} => {}", result.name);
        assert!(result.name.ends_with(&region_name));
        if discard_year.is_empty() {
            assert_eq!(result.discard_year, 0);
        } else {
            assert_eq!(result.discard_year, discard_year.parse::<u32>().unwrap());
        }
    }
}
