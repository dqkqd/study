package main

import "fmt"

func main() {
	nums := []int{1, 3, 8, 48, 10}
	ans := longestNiceSubarray(nums)
	fmt.Printf("%d\n", ans)
}

func longestNiceSubarray(nums []int) int {
	i, j, maxRange := 0, 1, 1

	counts := [32]int{}

	add := func(x int) {
		i := 0
		for x > 0 {
			counts[i] += x & 1
			i++
			x >>= 1
		}
	}

	rm := func(x int) {
		i := 0
		for x > 0 {
			counts[i] -= x & 1
			i++
			x >>= 1
		}
	}

	good := func() bool {
		for _, x := range counts {
			if x > 1 {
				return false
			}
		}
		return true
	}

	add(nums[0])

	for i < len(nums) && j < len(nums) {
		for !good() && i < len(nums) {
			rm(nums[i])
			i++
		}

		for good() && j < len(nums) {
			maxRange = max(maxRange, j-i)
			add(nums[j])
			j++
		}
	}

	if j == len(nums) && good() {
		maxRange = max(maxRange, j-i)
	}

	return maxRange
}
