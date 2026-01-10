from collections.abc import Iterable
from pathlib import Path


def example(day: int) -> Iterable[str]:
    input_dir = find_input_dir()
    path = input_dir / f"day{day:02}" / "example.txt"
    return path.open()


def actual(day: int) -> Iterable[str]:
    input_dir = find_input_dir()
    path = input_dir / f"day{day:02}" / "actual.txt"
    return path.open()


def find_input_dir() -> Path:
    path = Path.cwd()
    while path.parent != path:
        input_dir = path / "input"
        if input_dir.is_dir():
            return input_dir
        path = path.parent
    raise Exception("Cannot find input directory")
