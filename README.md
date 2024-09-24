[![Crates.io](https://img.shields.io/crates/v/region-cn?style=flat-square)](https://crates.io/crates/region-cn)
[![region-cn](https://github.com/bujnlc8/region-cn/actions/workflows/region-cn.yml/badge.svg)](https://github.com/bujnlc8/region-cn/actions/workflows/region-cn.yml)

# 根据 中国 6 位行政区划代码查询地区名称

数据来源于[https://www.mca.gov.cn/mzsj/xzqh/2023/202301xzqh.html](https://www.mca.gov.cn/mzsj/xzqh/2023/202301xzqh.html)，是目前官方最新的数据。

数据经过压缩处理，最新的版本可从[data/region.dat](./data/region.dat)下载。

由于行政区划经过多次调整，很多区划代码都已废弃，但可能还在使用，这种废弃的代码在最新的数据是查找不到的，比如`110103` 北京市崇文区，于 2010 年废弃。

因此从[https://zh.wikipedia.org/wiki/中华人民共和国行政区划代码](https://zh.wikipedia.org/wiki/%E4%B8%AD%E5%8D%8E%E4%BA%BA%E6%B0%91%E5%85%B1%E5%92%8C%E5%9B%BD%E8%A1%8C%E6%94%BF%E5%8C%BA%E5%88%92%E4%BB%A3%E7%A0%81)整理了历史数据，这可能是全网能找到的最全的记录了，可以从[data/region_full.dat](./data/region_full.dat)下载。

也提供了 MySQL 数据库 SQL 供下载 [data/region.sql](./data/region.sql)

`region*.dat`文件的数据结构如下:

![region-code.png](./region-code.png)

提供 2 种搜索方式，前缀树和文件搜索。

## 使用

```rust

use std::path::PathBuf;

use region_cn::region::Region;

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
    // 通过前缀树来搜索结果
    let result = region.search_with_trie("530925").unwrap();
    assert_eq!(result.name, "云南省临沧市双江拉祜族佤族布朗族傣族自治县");
}

```

## Install

```
[dependencies]
region-cn = "0.1"
```
