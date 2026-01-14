from dataclasses import dataclass, field
from typing import TextIO
from utils import actual, example


@dataclass(frozen=True)
class Point:
    x: int
    y: int

    def update(self) -> Point:
        return Point(self.x, self.y + 1)

    def split(self) -> tuple[Point, Point]:
        return Point(self.x - 1, self.y), Point(self.x + 1, self.y)

    def inbound(self, rows: int, cols: int) -> bool:
        return self.x >= 0 and self.x < cols and self.y >= 0 and self.y < rows


@dataclass()
class Grid:
    splitters: set[Point] = field(default_factory=set)
    start: Point | None = None
    beams: set[Point] = field(default_factory=set)
    split_time: int = 0
    rows: int = 0
    cols: int = 0

    def addline(self, line: str, row: int):
        self.rows = max(self.rows, row + 1)
        for col, c in enumerate(line.strip()):
            self.cols = max(self.cols, col + 1)
            match c:
                case "S":
                    self.start = Point(col, row)
                case "^":
                    self.splitters.add(Point(col, row))
                case _:
                    continue

    def updatepart1(self) -> bool:
        if not self.beams:
            return False
        self.beams = set(
            filter(
                lambda b: b.inbound(self.rows, self.cols),
                map(lambda b: b.update(), self.beams),
            )
        )
        return True

    def splitpart1(self):
        for s in self.splitters:
            if s in self.beams:
                self.split_time += 1
                b1, b2 = s.split()
                self.beams.remove(s)
                assert b1 not in self.splitters
                assert b2 not in self.splitters
                self.beams.add(b1)
                self.beams.add(b2)

    def solve_part1(self):
        assert self.start is not None
        self.beams.add(self.start)
        while self.updatepart1():
            self.splitpart1()
        return self.split_time

    def solve_part2(self) -> int:
        assert self.start is not None
        times: list[list[int]] = [[0] * self.cols for _ in range(self.rows)]
        times[self.start.y][self.start.x] = 1
        for y in range(1, self.rows):
            for x in range(self.cols):
                p = Point(x, y)
                if p in self.splitters:
                    continue
                times[y][x] += times[y - 1][x]
                p = Point(x - 1, y)
                if p in self.splitters:
                    times[y][x] += times[y - 1][x - 1]
                p = Point(x + 1, y)
                if p in self.splitters:
                    times[y][x] += times[y - 1][x + 1]
        return sum(times[self.rows - 1])


def solve(lines: TextIO):
    grid = Grid()
    for row, line in enumerate(lines):
        grid.addline(line, row)
    print(grid.solve_part1(), grid.solve_part2())


def main():
    solve(example(7))
    solve(actual(7))


if __name__ == "__main__":
    main()
