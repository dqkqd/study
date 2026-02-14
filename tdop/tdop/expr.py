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
    lhs: Expr | None
    rhs: Expr

    @t.override
    def __repr__(self) -> str:
        if self.lhs is None:
            return f"(+ {self.rhs})"
        return f"(+ {self.lhs} {self.rhs})"

    @t.override
    def evalulate(self) -> int:
        if self.lhs is None:
            return self.rhs.evalulate()
        return self.lhs.evalulate() + self.rhs.evalulate()


@t.final
@dataclass(kw_only=True)
class SubExpr(Expr):
    lhs: Expr | None
    rhs: Expr

    @t.override
    def __repr__(self) -> str:
        if self.lhs is None:
            return f"(- {self.rhs})"
        return f"(- {self.lhs} {self.rhs})"

    @t.override
    def evalulate(self) -> int:
        if self.lhs is None:
            return -self.rhs.evalulate()
        return self.lhs.evalulate() - self.rhs.evalulate()
