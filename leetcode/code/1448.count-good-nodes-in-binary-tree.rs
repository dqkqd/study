// Definition for a binary tree node.
// #[derive(Debug, PartialEq, Eq)]
// pub struct TreeNode {
//   pub val: i32,
//   pub left: Option<Rc<RefCell<TreeNode>>>,
//   pub right: Option<Rc<RefCell<TreeNode>>>,
// }
//
// impl TreeNode {
//   #[inline]
//   pub fn new(val: i32) -> Self {
//     TreeNode {
//       val,
//       left: None,
//       right: None
//     }
//   }
// }
use std::cell::RefCell;
use std::rc::Rc;

fn dfs(node: &Option<Rc<RefCell<TreeNode>>>, max: i32, good: &mut i32) {
    if let Some(node) = node {
        let node_val = node.borrow().val;
        if node_val >= max {
            *good += 1;
        }
        dfs(&node.borrow().left, max.max(node_val), good);
        dfs(&node.borrow().right, max.max(node_val), good);
    }
}

impl Solution {
    pub fn good_nodes(root: Option<Rc<RefCell<TreeNode>>>) -> i32 {
        let mut good = 0;
        dfs(&root, i32::MIN, &mut good);
        good
    }
}
