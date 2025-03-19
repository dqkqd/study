package main

import "fmt"

func main() {
	// nums := []int{0, 1, 1, 1, 0, 0}
	nums := []int{0, 1, 1, 1}
	fmt.Println(minOperations(nums))
}

func minOperations(nums []int) int {
	ans := 0
	i := 0
	for i < len(nums)-2 {
		if nums[i] != 1 {
			ans += 1
			nums[i] = 0
			nums[i+1] = 1 - nums[i+1]
			nums[i+2] = 1 - nums[i+2]
		}
		i++
	}
	if nums[len(nums)-1] == 0 || nums[len(nums)-2] == 0 {
		return -1
	}

	return ans
}
