from abc import ABC, abstractmethod
from dataclasses import dataclass
import typing as t


class Expr(ABC):
    @t.override
    @abstractmethod
    def __repr__(self) -> str: ...

    @abstractmethod
    def evalulate(self) -> int: ...


@t.final
@dataclass(kw_only=True)
class LiteralExpr(Expr):
    value: int

    @t.override
    def __repr__(self) -> str:
        return str(self.value)

    @t.override
    def evalulate(self) -> int:
        return self.value


@t.final
@dataclass(kw_only=True)
class AddExpr(Expr):
    first: Expr
    second: Expr | None

    @t.override
    def __repr__(self) -> str:
        if self.second is None:
            return f"(+ {self.first})"
        return f"(+ {self.first} {self.second})"

    @t.override
    def evalulate(self) -> int:
        first = self.first.evalulate()
        second = self.second.evalulate() if self.second is not None else 0
        return first + second


@t.final
@dataclass(kw_only=True)
class SubExpr(Expr):
    first: Expr
    second: Expr | None

    @t.override
    def __repr__(self) -> str:
        if self.second is None:
            return f"(- {self.first})"
        return f"(- {self.first} {self.second})"

    @t.override
    def evalulate(self) -> int:
        first = self.first.evalulate()
        if self.second is None:
            return -first
        return first - self.second.evalulate()
