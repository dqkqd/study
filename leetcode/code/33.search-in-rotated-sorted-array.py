import bisect


class Solution:
    def search(self, nums: list[int], target: int) -> int:
        lo = 0
        hi = len(nums)
        while lo < hi:
            mi = (lo + hi) // 2
            if nums[mi] > nums[-1]:
                lo = mi + 1
            else:
                hi = mi

        p = lo
        index = bisect.bisect_left(nums, target, lo=0, hi=p)
        if index >= 0 and index < p and nums[index] == target:
            return index
        index = bisect.bisect_left(nums, target, lo=p)
        if index >= p and index < len(nums) and nums[index] == target:
            return index
        return -1
