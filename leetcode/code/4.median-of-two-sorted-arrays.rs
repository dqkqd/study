impl Solution {
    pub fn find_median_sorted_arrays(nums1: Vec<i32>, nums2: Vec<i32>) -> f64 {
        let n1 = nums1.len();
        let n2 = nums2.len();
        let range = (n1 + n2 - 1) / 2;

        let solve = |nums1: &[i32], nums2: &[i32]| -> Option<usize> {
            let mut low = 0i32;
            let mut high = (nums1.len() - 1) as i32;
            let get = |i: i32| {
                if i < 0 {
                    i32::MIN
                } else {
                    *nums2.get(i as usize).unwrap_or(&i32::MAX)
                }
            };
            loop {
                if low > high {
                    break None;
                }
                let mid = (low + high) / 2;
                let l = mid;
                if l > range as i32 {
                    high = mid - 1;
                    continue;
                }
                let x = nums1[mid as usize];
                let l = (range as i32 - l) as i32;
                let lower = get(l - 1);
                let higher = get(l);
                if x < lower {
                    low = mid + 1;
                } else if x > higher {
                    high = mid - 1;
                } else {
                    break Some(mid as usize);
                }
            }
        };

        if let Some(mid) = solve(&nums1, &nums2) {
            if (n1 + n2) % 2 == 1 {
                nums1[mid] as f64
            } else {
                let mid2 = range - mid;
                let y1 = nums1.get(mid + 1).unwrap_or(&i32::MAX);
                let y2 = nums2.get(mid2).unwrap_or(&i32::MAX);
                let x = nums1[mid];
                let y = *y1.min(y2);
                (x + y) as f64 / 2.0
            }
        } else if let Some(mid) = solve(&nums2, &nums1) {
            if (n1 + n2) % 2 == 1 {
                nums2[mid] as f64
            } else {
                let mid2 = (n1 + n2) / 2 - mid - 1;
                let y2 = nums2.get(mid + 1).unwrap_or(&i32::MAX);
                let y1 = nums1.get(mid2).unwrap_or(&i32::MAX);
                let x = nums2[mid];
                let y = *y1.min(y2);
                (x + y) as f64 / 2.0
            }
        } else {
            unreachable!()
        }
    }
}
