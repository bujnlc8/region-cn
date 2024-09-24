//! 前缀树实现，每个节点代表2位地区代码
use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::RegionItem;

#[derive(Debug, Clone)]
pub struct RegionNameItem {
    text: String,
    discard_year: u32,
}

impl Default for RegionNameItem {
    fn default() -> Self {
        Self {
            text: Default::default(),
            discard_year: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegionNode {
    children: HashMap<String, RegionNode>,
    item: RegionNameItem,
}

impl RegionNode {
    fn new(item: RegionNameItem) -> Self {
        RegionNode {
            children: HashMap::new(),
            item,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegionTrie {
    root: RegionNode,
}

impl RegionTrie {
    pub fn new() -> Self {
        RegionTrie {
            root: RegionNode::new(RegionNameItem::default()),
        }
    }

    /// 插入地区码和地区
    pub fn insert(&mut self, key: String, value: String, discard_year: u32) {
        let mut node = &mut self.root;
        let trimed_key = key.trim_end_matches("00");
        for (i, s) in trimed_key
            .chars()
            .collect::<Vec<_>>()
            .chunks(2)
            .map(|chunk| chunk.iter().collect::<String>())
            .enumerate()
        {
            node = node.children.entry(s).or_insert_with(|| {
                let mut text = String::new();
                if i + 1 == trimed_key.len() / 2 {
                    text = value.clone();
                    RegionNode::new(RegionNameItem { text, discard_year })
                } else {
                    RegionNode::new(RegionNameItem {
                        text,
                        discard_year: 0,
                    })
                }
            });
        }
    }

    // 搜索地区码
    pub fn search(&self, region_code: &str) -> Result<RegionItem> {
        let mut node = &self.root;
        let mut res: Vec<RegionNameItem> = Vec::new();
        for s in region_code
            .chars()
            .collect::<Vec<_>>()
            .chunks(2)
            .map(|chunk| chunk.iter().collect::<String>())
        {
            if s == "00" {
                continue;
            }
            match node.children.get(&s) {
                Some(next_node) => {
                    node = next_node;
                    res.push(next_node.item.clone());
                }
                None => {
                    break;
                }
            }
        }
        if res.is_empty() {
            return Err(anyhow!("cannot find record"));
        }
        let region_slice: Vec<String> = res.iter().map(|x| x.text.clone()).collect();
        Ok(RegionItem {
            region_code: region_code.to_string(),
            name: region_slice.join(""),
            region_slice,
            discard_year: res.last().unwrap().discard_year,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_trie() {
        let mut tree = RegionTrie::new();
        tree.insert(String::from("110000"), String::from("北京市"), 0);
        tree.insert(String::from("110101"), String::from("东城区"), 0);
        tree.insert(String::from("110102"), String::from("西城区"), 0);
        tree.insert(String::from("110103"), String::from("崇文区"), 2010);
        tree.insert(String::from("110105"), String::from("朝阳区"), 0);
        tree.insert(String::from("130000"), String::from("河北省"), 0);
        tree.insert(String::from("130100"), String::from("石家庄市"), 0);
        tree.insert(String::from("130102"), String::from("长安区"), 0);
        tree.insert(String::from("130104"), String::from("桥西区"), 0);
        assert_eq!(tree.search("110000").unwrap().name, String::from("北京市"));
        assert_eq!(
            tree.search("110101").unwrap().name,
            String::from("北京市东城区")
        );
        assert_eq!(tree.search("110101").unwrap().discard_year, 0,);
        assert_eq!(
            tree.search("110103").unwrap().name,
            String::from("北京市崇文区")
        );
        assert_eq!(tree.search("110103").unwrap().discard_year, 2010,);
        assert_eq!(tree.search("130000").unwrap().name, String::from("河北省"));
        assert_eq!(
            tree.search("130100").unwrap().name,
            String::from("河北省石家庄市")
        );
        assert_eq!(
            tree.search("130102").unwrap().name,
            String::from("河北省石家庄市长安区")
        );
    }
}
