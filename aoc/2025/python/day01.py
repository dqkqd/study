from dataclasses import dataclass
from enum import StrEnum
from typing import TextIO
from utils import actual, example


class Direction(StrEnum):
    Left = "L"
    Right = "R"


@dataclass()
class Rotation:
    direction: Direction
    distance: int

    @classmethod
    def from_str(cls, s: str) -> Rotation:
        distance = int(s[1:])
        match s[0]:
            case "L":
                return Rotation(direction=Direction.Left, distance=distance)
            case "R":
                return Rotation(direction=Direction.Right, distance=distance)
            case _:
                raise ValueError(s)


@dataclass()
class Dial:
    position: int
    seen_zero: int
    stop_at_zero: int

    def update(self, rotation: Rotation):
        match rotation.direction:
            case Direction.Left:
                if self.position == 0:
                    self.position -= rotation.distance
                    self.seen_zero += (-self.position) // 100
                else:
                    self.position -= rotation.distance
                    if self.position <= 0:
                        self.seen_zero += (-self.position) // 100 + 1
            case Direction.Right:
                self.position += rotation.distance
                self.seen_zero += self.position // 100

        self.position %= 100
        self.position += 100
        self.position %= 100

        if self.position == 0:
            self.stop_at_zero += 1


def solve(lines: TextIO):
    dial = Dial(50, 0, 0)
    for line in lines:
        rotation = Rotation.from_str(line)
        dial.update(rotation)
    print(dial)


def main():
    solve(example(1))
    solve(actual(1))


if __name__ == "__main__":
    main()
