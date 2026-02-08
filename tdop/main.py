from abc import ABC
from dataclasses import dataclass
from typing import final, override
from collections.abc import Iterator


@dataclass
class Tokenizer:
    expression: str

    def iter(self) -> Iterator[Token]:
        for tok in self.expression:
            match tok:
                case tok if tok.isdigit():
                    yield LiteralToken(int(tok))
                case "+":
                    yield AddToken()
                case "-":
                    yield SubToken()
                case "*":
                    yield MulToken()
                case " ":
                    continue
                case _:
                    raise NotImplementedError(tok)
        yield EndToken()

    def stream(self) -> Stream:
        iter = self.iter()
        cur = next(iter)
        return Stream(cur, iter)


@dataclass
class Stream:
    token: Token
    iter: Iterator[Token]

    def next(self) -> None:
        self.token = next(self.iter)

    def done(self) -> bool:
        return isinstance(self.token, EndToken)


class Token(ABC):
    lbp: int

    def nud(self) -> int:
        raise NotImplementedError(self.__class__)

    def led(self, left: int, stream: Stream) -> int:  # pyright: ignore[reportUnusedParameter]
        raise NotImplementedError(self.__class__)


@final
@dataclass
class LiteralToken(Token):
    value: int
    lbp: int = 1

    @override
    def nud(self) -> int:
        return self.value


@final
@dataclass
class AddToken(Token):
    lbp: int = 10

    @override
    def led(self, left: int, stream: Stream) -> int:
        right = eval_expression(self.lbp, stream)
        return left + right


@final
@dataclass
class SubToken(Token):
    lbp: int = 10

    @override
    def led(self, left: int, stream: Stream) -> int:
        right = eval_expression(self.lbp, stream)
        return left - right


@final
@dataclass
class MulToken(Token):
    lbp: int = 20

    @override
    def led(self, left: int, stream: Stream) -> int:
        right = eval_expression(self.lbp, stream)
        return left * right


@final
@dataclass
class EndToken(Token):
    lbp: int = 0


def eval_expression(rbp: int, stream: Stream) -> int:
    left = stream.token.nud()
    stream.next()
    while rbp < stream.token.lbp:
        t = stream.token
        stream.next()
        left = t.led(left, stream)
    return left


def evaluate(expression: str) -> None:
    tokenizer = Tokenizer(expression)
    result = eval_expression(0, tokenizer.stream())
    print(f"{expression} = {result}")


def main():
    evaluate("1 + 2 * 3 * 5")
    evaluate("1 + 2 * 3 * 4 + 5")
    evaluate("1 - 2 * 3 * 4 + 5")


if __name__ == "__main__":
    main()
