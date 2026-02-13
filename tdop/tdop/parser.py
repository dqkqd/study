from tdop.expr import AddExpr, Expr, LiteralExpr, SubExpr
from tdop.token import Token, TokenType, Tokenizer
import typing as t


type PrefixParserFn = t.Callable[[Tokenizer, Token], Expr]


def parse(program: str) -> Expr:
    tokenizer = Tokenizer.from_str(program)
    return parse_expr(tokenizer)


def parse_expr(tokenizer: Tokenizer) -> Expr:
    token = tokenizer.next()
    prefix = PrefixParser.get(token.token_type)
    lhs = prefix(tokenizer, token)
    return lhs


class PrefixParser:
    inner: t.ClassVar[dict[TokenType, PrefixParserFn]] = {}

    @classmethod
    def get(cls, token_type: TokenType) -> PrefixParserFn:
        if token_type not in cls.inner:
            raise KeyError(f"{token_type} has not been registered")
        return cls.inner[token_type]

    @classmethod
    def register(cls, token_type: TokenType):
        def inner(fn: PrefixParserFn):
            if token_type in cls.inner:
                raise KeyError(f"{token_type} is already exist in prefix parser")
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
    return AddExpr(first=rhs, second=None)


@PrefixParser.register(TokenType.Sub)
def _(tokenizer: Tokenizer, token: Token) -> SubExpr:
    assert token.token_type == TokenType.Sub
    rhs = parse_expr(tokenizer)
    return SubExpr(first=rhs, second=None)
