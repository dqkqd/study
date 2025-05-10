// Definition for singly-linked list.
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
    pub fn remove_nth_from_end(mut head: Option<Box<ListNode>>, n: i32) -> Option<Box<ListNode>> {
        let mut m = 0;
        let mut h = &head;
        while let Some(v) = h {
            m += 1;
            h = &v.next;
        }
        let mut n = m - n;
        if n == 0 {
            match head {
                Some(next) => next.next,
                None => None,
            }
        } else if n < 0 {
            head
        } else {
            let mut h = &mut head;
            while let Some(v) = h {
                n -= 1;
                if n == 0 {
                    let s = v.next.take();
                    if let Some(s) = s {
                        v.next = s.next;
                    }
                    break;
                }
                h = &mut v.next;
            }
            head
        }
    }
}
