use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::RegionItem;

#[derive(Debug, Clone)]
pub struct RegionNode {
    children: HashMap<String, RegionNode>,
    text: String,
}

impl RegionNode {
    fn new(text: String) -> Self {
        RegionNode {
            children: HashMap::new(),
            text,
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
            root: RegionNode::new(String::new()),
        }
    }

    /// 插入地区码和地区
    pub fn insert(&mut self, key: String, value: String) {
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
                }
                RegionNode::new(text)
            });
        }
    }

    // 搜索地区码
    pub fn search(&self, region_code: &str) -> Result<RegionItem> {
        let mut node = &self.root;
        let mut res: Vec<String> = Vec::new();
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
                    res.push(next_node.text.clone());
                }
                None => {
                    break;
                }
            }
        }
        if res.is_empty() {
            return Err(anyhow!("cannot find record"));
        }
        Ok(RegionItem {
            region_code: region_code.to_string(),
            name: res.join(""),
            region_slice: res,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_trie() {
        let mut tree = RegionTrie::new();
        tree.insert(String::from("110000"), String::from("北京市"));
        tree.insert(String::from("110101"), String::from("东城区"));
        tree.insert(String::from("110102"), String::from("西城区"));
        tree.insert(String::from("110105"), String::from("朝阳区"));
        tree.insert(String::from("130000"), String::from("河北省"));
        tree.insert(String::from("130100"), String::from("石家庄市"));
        tree.insert(String::from("130102"), String::from("长安区"));
        tree.insert(String::from("130104"), String::from("桥西区"));
        assert_eq!(tree.search("110000").unwrap().name, String::from("北京市"));
        assert_eq!(
            tree.search("110101").unwrap().name,
            String::from("北京市东城区")
        );
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
