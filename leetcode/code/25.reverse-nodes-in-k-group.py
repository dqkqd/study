# # Definition for singly-linked list.
# from typing import Optional, final
#
#
# @final
# class ListNode:
#     def __init__(self, val: int = 0, next: Optional["ListNode"] = None):
#         self.val = val
#         self.next = next


class Solution:
    def reverseKGroup(self, head: Optional[ListNode], k: int) -> Optional[ListNode]:
        n = 0
        cur = head
        while cur is not None:
            n += 1
            cur = cur.next

        cur: Optional[ListNode] = None
        next = head

        for chunk in range(n // k):
            prev_tail = cur
            next_tail = next
            #
            # cur = next
            # if next is not None:
            #     next = next.next

            for _ in range(k):
                if next is None:
                    raise ValueError("invalid next")

                next_next = next.next
                next.next = cur
                cur = next
                next = next_next

            if chunk == 0:
                head = cur

            if prev_tail is not None:
                prev_tail.next = cur
            cur = next_tail
            if cur is not None:
                cur.next = next

        return head
