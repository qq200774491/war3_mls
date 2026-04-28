use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use mlua::prelude::*;

use super::{ProfileData, ProfileNode};

struct TreeNode {
    count: u64,
    self_count: u64,
    children: HashMap<String, TreeNode>,
}

impl TreeNode {
    fn new() -> Self {
        Self {
            count: 0,
            self_count: 0,
            children: HashMap::new(),
        }
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub(crate) struct ProfilerState {
    window_seconds: i64,
    max_depth: usize,
    buckets: HashMap<i64, TreeNode>,
    current_key: i64,
    total_samples: u64,
    name_cache: HashMap<String, String>,
    merged_cache: Option<ProfileData>,
    merged_cache_key: i64,
    stack_buf: Vec<String>,
}

impl ProfilerState {
    pub fn new() -> Self {
        Self {
            window_seconds: 15,
            max_depth: 32,
            buckets: HashMap::new(),
            current_key: 0,
            total_samples: 0,
            name_cache: HashMap::new(),
            merged_cache: None,
            merged_cache_key: -1,
            stack_buf: Vec::with_capacity(32),
        }
    }

    pub fn configure(&mut self, window_seconds: i32, max_depth: usize) {
        self.window_seconds = window_seconds as i64;
        self.max_depth = max_depth;
    }

    pub fn record_sample(&mut self, lua: &Lua) {
        let now = now_secs();

        if now != self.current_key {
            self.current_key = now;
            let cutoff = now - self.window_seconds;
            self.buckets.retain(|k, _| *k >= cutoff);
            self.buckets.entry(now).or_insert_with(TreeNode::new);
            self.merged_cache = None;
        }

        self.stack_buf.clear();
        for level in 0..self.max_depth {
            let debug = match lua.inspect_stack(level) {
                Some(d) => d,
                None => break,
            };

            let source = debug.source();
            let is_c = &*source.what == "C";

            let id = if is_c {
                let names = debug.names();
                let cname = names.name.as_deref().unwrap_or("[C]");
                let id = format!("[C]:{}", cname);
                if !self.name_cache.contains_key(&id) {
                    self.name_cache.insert(id.clone(), cname.to_string());
                }
                id
            } else {
                let short_src = source.short_src.as_deref().unwrap_or("?");
                let line_def = source.line_defined.unwrap_or(0);
                let id = format!("{}:{}", short_src, line_def);
                if !self.name_cache.contains_key(&id) {
                    let names = debug.names();
                    let display = names
                        .name
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| id.clone());
                    self.name_cache.insert(id.clone(), display);
                }
                id
            };

            self.stack_buf.push(id);
        }

        let n = self.stack_buf.len();
        if n == 0 {
            return;
        }

        let bucket = self.buckets.get_mut(&self.current_key).unwrap();
        bucket.count += 1;

        let mut node = bucket;
        for i in (0..n).rev() {
            let id = self.stack_buf[i].clone();
            node = node.children.entry(id).or_insert_with(TreeNode::new);
            node.count += 1;
        }
        node.self_count += 1;
        self.total_samples += 1;
    }

    pub fn reset(&mut self) {
        self.buckets.clear();
        self.current_key = 0;
        self.total_samples = 0;
        self.name_cache.clear();
        self.merged_cache = None;
        self.merged_cache_key = -1;
    }

    pub fn to_profile_data(&mut self, running: bool) -> ProfileData {
        let now = now_secs();

        if let Some(ref cached) = self.merged_cache {
            if self.merged_cache_key == now {
                return ProfileData {
                    running,
                    ..cached.clone()
                };
            }
        }

        let cutoff = now - self.window_seconds;
        self.buckets.retain(|k, _| *k >= cutoff);

        let mut merged = TreeNode::new();
        let mut active_count = 0u64;
        let mut active_samples = 0u64;

        for root in self.buckets.values() {
            active_count += 1;
            active_samples += root.count;
            merge_tree(&mut merged, root);
        }

        let result = ProfileData {
            root: self.tree_to_node("(root)", &merged),
            total_samples: active_samples,
            window: self.window_seconds as u64,
            bucket_count: active_count,
            running,
        };

        self.merged_cache = Some(result.clone());
        self.merged_cache_key = now;
        result
    }

    fn tree_to_node(&self, id: &str, node: &TreeNode) -> ProfileNode {
        let name = self
            .name_cache
            .get(id)
            .cloned()
            .unwrap_or_else(|| id.to_string());

        let mut children: Vec<ProfileNode> = node
            .children
            .iter()
            .map(|(cid, child)| self.tree_to_node(cid, child))
            .collect();
        children.sort_by(|a, b| b.count.cmp(&a.count));

        ProfileNode {
            id: id.to_string(),
            name,
            count: node.count,
            self_count: node.self_count,
            children,
        }
    }
}

fn merge_tree(dst: &mut TreeNode, src: &TreeNode) {
    dst.count += src.count;
    dst.self_count += src.self_count;
    for (id, schild) in &src.children {
        let dchild = dst.children.entry(id.clone()).or_insert_with(TreeNode::new);
        merge_tree(dchild, schild);
    }
}
