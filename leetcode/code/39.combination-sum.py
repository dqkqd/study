def backtrack(
    solutions: list[list[int]],
    vec: list[int],
    sum: int,
    target: int,
    candidates: list[int],
    candidate_idx: int,
):
    if sum == target:
        solutions.append(vec)
        return
    if candidate_idx >= len(candidates) or sum + candidates[candidate_idx] > target:
        return

    value = candidates[candidate_idx]

    backtrack(solutions, vec, sum, target, candidates, candidate_idx + 1)
    while sum < target:
        sum += value
        if sum > target:
            break
        vec = vec[:]
        vec.append(value)
        backtrack(
            solutions,
            vec,
            sum,
            target,
            candidates,
            candidate_idx + 1,
        )


class Solution:
    def combinationSum(self, candidates: list[int], target: int) -> list[list[int]]:
        solutions: list[list[int]] = []
        backtrack(solutions, [], 0, target, sorted(candidates), 0)
        return solutions
