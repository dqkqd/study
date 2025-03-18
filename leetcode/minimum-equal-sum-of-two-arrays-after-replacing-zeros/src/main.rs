pub fn min_sum(nums1: Vec<i32>, nums2: Vec<i32>) -> i64 {
    let zeroes1 = nums1.iter().filter(|n| n == &&0).count();
    let zeroes2 = nums2.iter().filter(|n| n == &&0).count();

    let min1: i64 = nums1.iter().map(|n| *n as i64).sum::<i64>() + zeroes1 as i64;
    let min2: i64 = nums2.iter().map(|n| *n as i64).sum::<i64>() + zeroes2 as i64;

    let answer = min1.max(min2);

    if answer > min1 && zeroes1 == 0 {
        return -1;
    }
    if answer > min2 && zeroes2 == 0 {
        return -1;
    }

    answer
}
fn main() {
    let result = min_sum(vec![3, 2, 0, 1, 0], vec![6, 5, 0]);

    println!("{}", result)
}
