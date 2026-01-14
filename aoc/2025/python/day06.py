from dataclasses import dataclass
from functools import reduce
from typing import TextIO
from utils import actual, example


def apply(num: list[int], op: str) -> int:
    value = 0
    match op.strip():
        case "+":
            value += sum(num)
        case "*":
            value += reduce(lambda x, y: x * y, num, initial=1)
        case _:
            raise ValueError(op)
    return value


@dataclass()
class Grid2:
    chars: list[str]
    ops: list[str]

    def update(self, line: str):
        if "+" in line or "*" in line:
            self.ops = line.strip().split()
        else:
            self.chars.append(line.strip("\n"))

    def to_grid(self) -> Grid:
        values: list[str] = []
        for line in self.chars:
            for i, v in enumerate(line):
                if i >= len(values):
                    values.append("")
                values[i] += v
        nums: list[list[int]] = [[]]
        for v in values:
            x = v.strip()
            if x == "":
                nums.append([])
            else:
                nums[-1].append(int(x))
        return Grid(nums, self.ops)


@dataclass()
class Grid:
    nums: list[list[int]]
    ops: list[str]

    def update(self, line: str):
        values = line.strip().split()
        if not self.nums:
            self.nums = [[] for _ in range(len(values))]
        for i, v in enumerate(values):
            try:
                self.nums[i].append(int(v))
            except ValueError:
                self.ops = values
                return

    def apply(self) -> int:
        value = 0
        for op, num in zip(self.ops, self.nums, strict=True):
            value += apply(num, op)
        return value


def solve(lines: TextIO):
    grid = Grid([], [])
    grid2 = Grid2([], [])
    for row, line in enumerate(lines):
        grid.update(line)
        grid2.update(line)
    print(grid.apply(), grid2.to_grid().apply())


def main():
    solve(example(6))
    solve(actual(6))


if __name__ == "__main__":
    main()
