// Definition for a binary tree node.
// #[derive(Debug, PartialEq, Eq)]
// pub struct TreeNode {
//     pub val: i32,
//     pub left: Option<Rc<RefCell<TreeNode>>>,
//     pub right: Option<Rc<RefCell<TreeNode>>>,
// }
//
// impl TreeNode {
//     #[inline]
//     pub fn new(val: i32) -> Self {
//         TreeNode {
//             val,
//             left: None,
//             right: None,
//         }
//     }
// }
use std::cell::RefCell;
use std::rc::Rc;

fn max_diameter(node: Option<Rc<RefCell<TreeNode>>>) -> (i32, i32) {
    match node {
        Some(node) => {
            let (left, left_height) = max_diameter(node.borrow().left.clone());
            let (right, right_height) = max_diameter(node.borrow().right.clone());
            let mid = left_height + right_height;
            (left.max(right).max(mid), left_height.max(right_height) + 1)
        }
        None => (0, 0),
    }
}

impl Solution {
    pub fn diameter_of_binary_tree(root: Option<Rc<RefCell<TreeNode>>>) -> i32 {
        max_diameter(root).0
    }
}
