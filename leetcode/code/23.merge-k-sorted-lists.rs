use std::{cmp::Ordering, collections::BinaryHeap};

// Definition for singly-linked list.
// #[derive(PartialEq, Eq, Clone, Debug)]
// pub struct ListNode {
//     pub val: i32,
//     pub next: Option<Box<ListNode>>,
// }
//
// impl ListNode {
//     #[inline]
//     fn new(val: i32) -> Self {
//         ListNode { next: None, val }
//     }
// }

impl Ord for ListNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.val.cmp(&self.val)
    }
}

impl PartialOrd for ListNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.val.partial_cmp(&self.val)
    }
}

impl Solution {
    pub fn merge_k_lists(lists: Vec<Option<Box<ListNode>>>) -> Option<Box<ListNode>> {
        let mut heap = BinaryHeap::from(lists);
        let mut vals = vec![];
        while let Some(head) = heap.pop() {
            if let Some(head) = head {
                vals.push(head.val);
                heap.push(head.next);
            }
        }

        let mut head = None;
        for val in vals.into_iter().rev() {
            head = Some(Box::new(ListNode { val, next: head }));
        }

        head
    }
}
