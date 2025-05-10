// Definition for singly-linked list.
// #[derive(PartialEq, Eq, Clone, Debug)]
// pub struct ListNode {
//   pub val: i32,
//   pub next: Option<Box<ListNode>>
// }
//
// impl ListNode {
//   #[inline]
//   fn new(val: i32) -> Self {
//     ListNode {
//       next: None,
//       val
//     }
//   }
// }
impl Solution {
    pub fn add_two_numbers(
        l1: Option<Box<ListNode>>,
        l2: Option<Box<ListNode>>,
    ) -> Option<Box<ListNode>> {
        let mut rem = 0;
        let mut l1 = l1;
        let mut l2 = l2;
        let mut sum = vec![];
        loop {
            if l1.is_none() && l2.is_none() {
                break;
            }
            let v1 = l1.unwrap_or(Box::new(ListNode { val: 0, next: None }));
            let v2 = l2.unwrap_or(Box::new(ListNode { val: 0, next: None }));
            let mut val = v1.val + v2.val + rem;
            if val > 9 {
                rem = 1;
                val %= 10;
            } else {
                rem = 0;
            }
            sum.push(val);

            l1 = v1.next;
            l2 = v2.next;
        }

        if rem != 0 {
            sum.push(rem);
        }

        let mut node = None;
        for s in sum.into_iter().rev() {
            node = Some(Box::new(ListNode { val: s, next: node }));
        }

        node
    }
}
