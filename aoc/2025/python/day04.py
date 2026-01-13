from dataclasses import dataclass
from typing import TextIO
from utils import actual, example


@dataclass()
class Grid:
    pos: set[tuple[int, int]]
    rows: int = 0
    cols: int = 0

    def update(self, line: str, row: int):
        self.rows = max(self.rows, row + 1)
        for col, c in enumerate(line.strip()):
            self.cols = max(self.cols, col + 1)
            if c == "@":
                self.pos.add((row, col))

    def adj(self, p: tuple[int, int]) -> list[tuple[int, int]]:
        ps: list[tuple[int, int]] = []
        for dx in [-1, 0, 1]:
            for dy in [-1, 0, 1]:
                if dx != 0 or dy != 0:
                    x, y = p[0] + dx, p[1] + dy
                    if x < 0 or x >= self.rows or y < 0 or y > self.cols:
                        continue
                    ps.append((x, y))
        return ps

    def can_remove(self) -> list[tuple[int, int]]:
        goods: list[tuple[int, int]] = []
        for p in self.pos:
            adj = self.adj(p)
            total = len(list(filter(lambda a: a in self.pos, adj)))
            if total < 4:
                goods.append(p)
        return goods

    def bfs_remove(self):
        papers = len(self.pos)
        while True:
            removing = self.can_remove()
            if not removing:
                break
            for p in removing:
                self.pos.remove(p)
        return papers - len(self.pos)


def solve(lines: TextIO):
    grid = Grid(set())
    for row, line in enumerate(lines):
        grid.update(line, row)
    print(len(grid.can_remove()), grid.bfs_remove())


def main():
    solve(example(4))
    solve(actual(4))


if __name__ == "__main__":
    main()
