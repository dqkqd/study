from abc import ABC, abstractmethod
from dataclasses import dataclass
import typing as t


class Expr(ABC):
    @t.override
    @abstractmethod
    def __repr__(self) -> str: ...

    @abstractmethod
    def evalulate(self) -> float: ...


@t.final
@dataclass(kw_only=True)
class LiteralExpr(Expr):
    value: int

    @t.override
    def __repr__(self) -> str:
        return str(self.value)

    @t.override
    def evalulate(self) -> float:
        return self.value


@t.final
@dataclass(kw_only=True)
class AddExpr(Expr):
    lhs: Expr | None
    rhs: Expr

    @t.override
    def __repr__(self) -> str:
        if self.lhs is None:
            return f"{self.rhs}"
        return f"(+ {self.lhs} {self.rhs})"

    @t.override
    def evalulate(self) -> float:
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
    def evalulate(self) -> float:
        if self.lhs is None:
            return -self.rhs.evalulate()
        return self.lhs.evalulate() - self.rhs.evalulate()


@t.final
@dataclass(kw_only=True)
class MulExpr(Expr):
    lhs: Expr
    rhs: Expr

    @t.override
    def __repr__(self) -> str:
        return f"(* {self.lhs} {self.rhs})"

    @t.override
    def evalulate(self) -> float:
        return self.lhs.evalulate() * self.rhs.evalulate()


@t.final
@dataclass(kw_only=True)
class DivExpr(Expr):
    lhs: Expr
    rhs: Expr

    @t.override
    def __repr__(self) -> str:
        return f"(/ {self.lhs} {self.rhs})"

    @t.override
    def evalulate(self) -> float:
        return self.lhs.evalulate() / self.rhs.evalulate()


@t.final
@dataclass(kw_only=True)
class PowExpr(Expr):
    base: Expr
    exponent: Expr

    @t.override
    def __repr__(self) -> str:
        return f"(^ {self.base} {self.exponent})"

    @t.override
    def evalulate(self) -> float:
        base = self.base.evalulate()
        exponent = self.exponent.evalulate()
        match base**exponent:
            case float(v) | int(v):
                return v
            case _:  # pyright: ignore[reportAny]
                raise ValueError(
                    f"Unsupported exponent value with base={base}, exponent={exponent}"
                )


@t.final
@dataclass(kw_only=True)
class ParenExpr(Expr):
    inner: Expr

    @t.override
    def __repr__(self) -> str:
        return f"{self.inner}"

    @t.override
    def evalulate(self) -> float:
        return self.inner.evalulate()
