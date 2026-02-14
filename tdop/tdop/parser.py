from tdop.expr import AddExpr, Expr, LiteralExpr, SubExpr
from tdop.token import Token, TokenType, Tokenizer
import typing as t


def parse(program: str) -> Expr:
    tokenizer = Tokenizer.from_str(program)
    return parse_expr(tokenizer)


def parse_expr(tokenizer: Tokenizer) -> Expr:
    token = tokenizer.next()

    prefix = PrefixParser.get(token.token_type)
    lhs = prefix(tokenizer, token)

    current_token = tokenizer.peek()
    if current_token is not None and current_token != Token.eof():
        infix = InfixParser.get(current_token.token_type)
        _ = tokenizer.next()
        lhs = infix(tokenizer, lhs, current_token)

    return lhs


type PrefixParserFn = t.Callable[[Tokenizer, Token], Expr]


class PrefixParser:
    inner: t.ClassVar[dict[TokenType, PrefixParserFn]] = {}

    @classmethod
    def get(cls, token_type: TokenType) -> PrefixParserFn:
        if token_type not in cls.inner:
            raise KeyError(f"{token_type} has not been registered to {cls.__name__}")
        return cls.inner[token_type]

    @classmethod
    def register(cls, token_type: TokenType):
        def inner(fn: PrefixParserFn):
            if token_type in cls.inner:
                raise KeyError(f"{token_type} is already exist in {cls.__name__}")
            cls.inner[token_type] = fn

        return inner


type InfixParserFn = t.Callable[[Tokenizer, Expr, Token], Expr]


class InfixParser:
    inner: t.ClassVar[dict[TokenType, InfixParserFn]] = {}

    @classmethod
    def get(cls, token_type: TokenType) -> InfixParserFn:
        if token_type not in cls.inner:
            raise KeyError(f"{token_type} has not been registered to {cls.__name__}")
        return cls.inner[token_type]

    @classmethod
    def register(cls, token_type: TokenType):
        def inner(fn: InfixParserFn):
            if token_type in cls.inner:
                raise KeyError(f"{token_type} is already exist in {cls.__name__}")
            cls.inner[token_type] = fn

        return inner


@PrefixParser.register(TokenType.Literal)
def _(_: Tokenizer, token: Token) -> LiteralExpr:
    assert token.token_type == TokenType.Literal
    assert token.value.isdigit()
    return LiteralExpr(value=int(token.value))


@PrefixParser.register(TokenType.Add)
def _(tokenizer: Tokenizer, token: Token) -> AddExpr:
    assert token.token_type == TokenType.Add
    rhs = parse_expr(tokenizer)
    return AddExpr(lhs=None, rhs=rhs)


@PrefixParser.register(TokenType.Sub)
def _(tokenizer: Tokenizer, token: Token) -> SubExpr:
    assert token.token_type == TokenType.Sub
    rhs = parse_expr(tokenizer)
    return SubExpr(lhs=None, rhs=rhs)


@InfixParser.register(TokenType.Add)
def _(tokenizer: Tokenizer, lhs: Expr, token: Token) -> AddExpr:
    assert token.token_type == TokenType.Add
    rhs = parse_expr(tokenizer)
    return AddExpr(lhs=lhs, rhs=rhs)


@InfixParser.register(TokenType.Sub)
def _(tokenizer: Tokenizer, lhs: Expr, token: Token) -> SubExpr:
    assert token.token_type == TokenType.Sub
    rhs = parse_expr(tokenizer)
    return SubExpr(lhs=lhs, rhs=rhs)
