from abc import ABC
from dataclasses import dataclass
import enum
import re


class TokenType(enum.StrEnum):
    Literal = enum.auto()
    Add = enum.auto()
    Sub = enum.auto()
    Mul = enum.auto()
    Div = enum.auto()
    LParen = enum.auto()
    RParen = enum.auto()
    Eof = enum.auto()


@dataclass(frozen=True)
class Token(ABC):
    token_type: TokenType
    value: str

    @classmethod
    def from_str(cls, s: str) -> Token:
        match s:
            case _ if s.isdigit():
                return Token.literal(s)
            case "+":
                return Token.add()
            case "-":
                return Token.sub()
            case "*":
                return Token.mul()
            case "/":
                return Token.div()
            case "(":
                return Token.lparen()
            case ")":
                return Token.rparen()
            case _:
                raise SyntaxError(f"bad token: {s}")

    @classmethod
    def literal(cls, value: str) -> Token:
        return Token(token_type=TokenType.Literal, value=value)

    @classmethod
    def add(cls) -> Token:
        return Token(token_type=TokenType.Add, value="+")

    @classmethod
    def sub(cls) -> Token:
        return Token(token_type=TokenType.Sub, value="-")

    @classmethod
    def mul(cls) -> Token:
        return Token(token_type=TokenType.Mul, value="*")

    @classmethod
    def div(cls) -> Token:
        return Token(token_type=TokenType.Div, value="/")

    @classmethod
    def lparen(cls) -> Token:
        return Token(token_type=TokenType.LParen, value="(")

    @classmethod
    def rparen(cls) -> Token:
        return Token(token_type=TokenType.RParen, value=")")

    @classmethod
    def eof(cls) -> Token:
        return Token(token_type=TokenType.Eof, value="")


@dataclass
class Tokenizer:
    tokens: list[Token]

    @classmethod
    def from_str(cls, program: str) -> Tokenizer:
        raw_tokens: list[str] = re.findall(r"\d+|[+\-*/()]", program)
        tokens = [Token.from_str(s) for s in raw_tokens]
        tokens.append(Token.eof())
        return Tokenizer(tokens=list(reversed(tokens)))

    def next(self):
        return self.tokens.pop()

    def peek(self) -> Token | None:
        if self.tokens:
            return self.tokens[len(self.tokens) - 1]
        return None

    def next_expect(self, expect: TokenType):
        token = self.next()
        if token.token_type != expect:
            raise ValueError(f"Expected {expect}, got {token.token_type}")
