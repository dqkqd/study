from dataclasses import dataclass
from typing import TextIO
from utils import actual, example


@dataclass()
class State:
    values: list[int]
    need: int
    battery: int = 0

    @classmethod
    def from_str(cls, s: str, need: int) -> State:
        values = list(map(int, s))
        return State(values, need)

    def update(self):
        n = len(self.values)
        v = max(self.values[: n - self.need + 1])
        self.battery *= 10
        self.battery += v
        index = self.values.index(v)
        self.values = self.values[index + 1 :]
        self.need -= 1

    def solve(self):
        while self.need > 0:
            self.update()


def solve(lines: TextIO):
    s1 = 0
    s2 = 0
    for line in lines:
        b1 = State.from_str(line.strip(), 2)
        b2 = State.from_str(line.strip(), 12)

        b1.solve()
        b2.solve()

        s1 += b1.battery
        s2 += b2.battery

    print(s1, s2)


def main():
    solve(example(3))
    solve(actual(3))


if __name__ == "__main__":
    main()
