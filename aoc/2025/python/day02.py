from dataclasses import dataclass
from typing import TextIO
from utils import actual, example


def part1_invalid(s: int) -> bool:
    value = str(s)
    n = len(value) // 2
    return value[:n] == value[n:]


def part2_invalid(s: int) -> bool:
    value = str(s)
    n = len(value)

    for i in range(1, n // 2 + 1):
        if n % i != 0:
            continue

        current = i
        good = True
        while current < n:
            if value[current : current + i] != value[:i]:
                good = False
                break
            current += i

        if good:
            return True

    return False


@dataclass()
class Range:
    lower: int
    upper: int

    @classmethod
    def from_str(cls, s: str) -> Range:
        s = s.strip()
        ranges = s.split("-")
        return Range(lower=int(ranges[0]), upper=int(ranges[1]))


def solve(lines: TextIO):
    line = next(lines).strip()

    part1_sum = 0
    part2_sum = 0
    for s in line.split(","):
        r = Range.from_str(s)
        for v in range(r.lower, r.upper + 1):
            if part1_invalid(v):
                part1_sum += v
            if part2_invalid(v):
                part2_sum += v

    print(part1_sum, part2_sum)


def main():
    solve(example(2))
    solve(actual(2))


if __name__ == "__main__":
    main()
