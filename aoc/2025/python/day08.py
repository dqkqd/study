from dataclasses import dataclass
from itertools import product
from typing import TextIO
from utils import actual, example


@dataclass(frozen=True)
class Point:
    index: int
    x: int
    y: int
    z: int

    @classmethod
    def from_str(cls, s: str, index: int) -> Point:
        x, y, z = list(map(int, s.split(",")))
        return Point(index, x, y, z)

    def dist(self, other: Point) -> int:
        return (
            (self.x - other.x) ** 2 + (self.y - other.y) ** 2 + (self.z - other.z) ** 2
        )


class Union:
    def __init__(self, n: int) -> None:
        self.ps = list(range(n))
        self.count = [1] * n
        self.cluster = n

    def merge(self, x: int, y: int):
        px = self.parent(x)
        py = self.parent(y)
        if px == py:
            return
        self.cluster -= 1
        self.ps[px] = py
        self.count[py] += self.count[px]

    def parent(self, x: int) -> int:
        while x != self.ps[self.ps[x]]:
            x = self.ps[self.ps[x]]
        return x


def solve(lines: TextIO, sz: int):
    points: list[Point] = []
    for index, line in enumerate(lines):
        points.append(Point.from_str(line.strip(), index))
    pairs = list(
        map(
            lambda ps: (ps[0].dist(ps[1]), ps[0].index, ps[1].index),
            filter(lambda ps: ps[0].index > ps[1].index, product(points, points)),
        )
    )
    pairs = sorted(pairs, key=lambda p: p[0])
    union = Union(len(points))
    for _, x, y in pairs[:sz]:
        union.merge(x, y)
    parents = set(union.parent(x) for x in range(len(points)))
    counts = sorted((union.count[x] for x in parents), reverse=True)
    a, b, c = counts[:3]
    print(a * b * c)

    for _, x, y in pairs[sz:]:
        union.merge(x, y)
        if union.cluster == 1:
            print(points[x].x * points[y].x)
            break


def main():
    solve(example(8), 10)
    solve(actual(8), 1000)


if __name__ == "__main__":
    main()
