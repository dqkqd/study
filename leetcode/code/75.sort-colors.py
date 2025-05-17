class Solution:
    def sortColors(self, nums: list[int]) -> None:
        """
        Do not return anything, modify nums in-place instead.
        """
        n = len(nums)
        i, i0, i2 = 0, 0, n - 1
        while i0 < i2 and i <= i2:
            match nums[i]:
                case 0:
                    nums[i], nums[i0] = nums[i0], nums[i]
                    i += 1
                    i0 += 1
                case 1:
                    i += 1
                case 2:
                    nums[i], nums[i2] = nums[i2], nums[i]
                    i2 -= 1
                case _:
                    raise ValueError
