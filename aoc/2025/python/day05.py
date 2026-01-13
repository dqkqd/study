from dataclasses import dataclass
from typing import TextIO
from utils import actual, example


@dataclass()
class Range:
    low: int
    high: int

    @classmethod
    def from_str(cls, s: str) -> Range:
        values = s.split("-")
        return Range(int(values[0]), int(values[1]))

    def merge(self, other: Range) -> Range | None:
        if self.low > other.low:
            return other.merge(self)
        if self.high >= other.low:
            return Range(self.low, max(self.high, other.high))
        return None


def solve(lines: TextIO):
    ranges: list[Range] = []
    for line in lines:
        line = line.strip()
        if line == "":
            break
        ranges.append(Range.from_str(line))
    count1 = 0
    for line in lines:
        v = int(line)
        for r in ranges:
            if v >= r.low and v <= r.high:
                count1 += 1
                break

    ranges = sorted(ranges, key=lambda r: -r.low)
    merged: list[Range] = []
    while ranges:
        r = ranges.pop()
        if not ranges:
            merged.append(r)
            break
        r2 = ranges.pop()
        r3 = r.merge(r2)
        if r3 is None:
            ranges.append(r2)
            merged.append(r)
        else:
            ranges.append(r3)

    count2 = 0
    for r in merged:
        count2 += r.high - r.low + 1
    print(count1, count2)


def main():
    solve(example(5))
    solve(actual(5))


if __name__ == "__main__":
    main()
