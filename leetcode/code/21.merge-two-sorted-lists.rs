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
impl Solution {
    pub fn merge_two_lists(
        mut list1: Option<Box<ListNode>>,
        mut list2: Option<Box<ListNode>>,
    ) -> Option<Box<ListNode>> {
        let mut sorted = vec![];
        loop {
            match (&list1, &list2) {
                (None, None) => break,
                (None, Some(a)) => {
                    sorted.push(a.val);
                    list2 = a.next.clone();
                }
                (Some(a), None) => {
                    sorted.push(a.val);
                    list1 = a.next.clone();
                }
                (Some(a), Some(b)) => {
                    if a.val < b.val {
                        sorted.push(a.val);
                        list1 = a.next.clone();
                    } else {
                        sorted.push(b.val);
                        list2 = b.next.clone();
                    }
                }
            }
        }

        let mut head = None;
        for v in sorted.into_iter().rev() {
            head = Some(Box::new(ListNode { val: v, next: head }));
        }

        head
    }
}
