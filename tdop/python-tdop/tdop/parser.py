from tdop.expr import (
    AddExpr,
    DivExpr,
    Expr,
    LiteralExpr,
    MulExpr,
    ParenExpr,
    PowExpr,
    SubExpr,
)
from tdop.token import Token, TokenType, Tokenizer
import typing as t


def parse(program: str) -> Expr:
    tokenizer = Tokenizer.from_str(program)
    return parse_expr(tokenizer, binding_power=0)


def parse_expr(tokenizer: Tokenizer, binding_power: int) -> Expr:
    lhs = PrefixParser.parse(tokenizer, tokenizer.next())

    token = tokenizer.peek()
    while (
        token is not None
        and token != Token.eof()
        and binding_power < InfixParser.binding_powers[token.token_type]
    ):
        # skip this operator
        _ = tokenizer.next()
        lhs = InfixParser.parse(tokenizer, lhs, token)
        token = tokenizer.peek()

    return lhs


class PrefixParser:
    type Fn = t.Callable[[Tokenizer, Token], Expr]

    parsers: t.ClassVar[dict[TokenType, Fn]] = {}

    binding_powers: dict[TokenType, int] = {
        TokenType.LParen: 0,
        TokenType.Literal: 10,
        TokenType.Add: 100,
        TokenType.Sub: 100,
    }

    @classmethod
    def parse(cls, tokenizer: Tokenizer, token: Token) -> Expr:
        if token.token_type not in cls.parsers:
            raise KeyError(
                f"{token.token_type} has not been registered to {cls.__name__}"
            )
        parser = cls.parsers[token.token_type]
        return parser(tokenizer, token)

    @classmethod
    def register(cls, token_type: TokenType):
        def inner(fn: PrefixParser.Fn):
            if token_type in cls.parsers:
                raise KeyError(f"{token_type} is already exist in {cls.__name__}")
            cls.parsers[token_type] = fn

        return inner


@PrefixParser.register(TokenType.Literal)
def _(_: Tokenizer, token: Token) -> LiteralExpr:
    assert token.token_type == TokenType.Literal
    assert token.value.isdigit()
    return LiteralExpr(value=int(token.value))


@PrefixParser.register(TokenType.Add)
def _(tokenizer: Tokenizer, token: Token) -> AddExpr:
    assert token.token_type == TokenType.Add
    rhs = parse_expr(tokenizer, PrefixParser.binding_powers[token.token_type])
    return AddExpr(lhs=None, rhs=rhs)


@PrefixParser.register(TokenType.Sub)
def _(tokenizer: Tokenizer, token: Token) -> SubExpr:
    assert token.token_type == TokenType.Sub
    rhs = parse_expr(tokenizer, PrefixParser.binding_powers[token.token_type])
    return SubExpr(lhs=None, rhs=rhs)


@PrefixParser.register(TokenType.LParen)
def _(tokenizer: Tokenizer, token: Token) -> ParenExpr:
    assert token.token_type == TokenType.LParen
    inner = parse_expr(tokenizer, PrefixParser.binding_powers[token.token_type])
    tokenizer.next_expect(TokenType.RParen)
    return ParenExpr(inner=inner)


class InfixParser:
    type Fn = t.Callable[[Tokenizer, Expr, Token], Expr]

    parsers: t.ClassVar[dict[TokenType, Fn]] = {}

    binding_powers: dict[TokenType, int] = {
        TokenType.RParen: 0,
        TokenType.Literal: 10,
        TokenType.Add: 20,
        TokenType.Sub: 20,
        TokenType.Mul: 30,
        TokenType.Div: 30,
        TokenType.Pow: 40,
    }

    @classmethod
    def parse(cls, tokenizer: Tokenizer, expr: Expr, token: Token) -> Expr:
        if token.token_type not in cls.parsers:
            raise KeyError(
                f"{token.token_type} has not been registered to {cls.__name__}"
            )
        parser = cls.parsers[token.token_type]
        return parser(tokenizer, expr, token)

    @classmethod
    def register(cls, token_type: TokenType):
        def inner(fn: InfixParser.Fn):
            if token_type in cls.parsers:
                raise KeyError(f"{token_type} is already exist in {cls.__name__}")
            cls.parsers[token_type] = fn

        return inner


@InfixParser.register(TokenType.Add)
def _(tokenizer: Tokenizer, lhs: Expr, token: Token) -> AddExpr:
    assert token.token_type == TokenType.Add
    rhs = parse_expr(tokenizer, InfixParser.binding_powers[token.token_type])
    return AddExpr(lhs=lhs, rhs=rhs)


@InfixParser.register(TokenType.Sub)
def _(tokenizer: Tokenizer, lhs: Expr, token: Token) -> SubExpr:
    assert token.token_type == TokenType.Sub
    rhs = parse_expr(tokenizer, InfixParser.binding_powers[token.token_type])
    return SubExpr(lhs=lhs, rhs=rhs)


@InfixParser.register(TokenType.Mul)
def _(tokenizer: Tokenizer, lhs: Expr, token: Token) -> MulExpr:
    assert token.token_type == TokenType.Mul
    rhs = parse_expr(tokenizer, InfixParser.binding_powers[token.token_type])
    return MulExpr(lhs=lhs, rhs=rhs)


@InfixParser.register(TokenType.Div)
def _(tokenizer: Tokenizer, lhs: Expr, token: Token) -> DivExpr:
    assert token.token_type == TokenType.Div
    rhs = parse_expr(tokenizer, InfixParser.binding_powers[token.token_type])
    return DivExpr(lhs=lhs, rhs=rhs)


@InfixParser.register(TokenType.Pow)
def _(tokenizer: Tokenizer, base: Expr, token: Token) -> PowExpr:
    assert token.token_type == TokenType.Pow
    exponent = parse_expr(tokenizer, InfixParser.binding_powers[token.token_type])
    return PowExpr(base=base, exponent=exponent)
