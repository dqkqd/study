import pytest

import tdop


@pytest.mark.parametrize(
    ["program", "expected_parsed_expr", "expected_evaluated_value"],
    [
        ("1", "1", 1),
        ("+1", "(+ 1)", 1),
        ("-1", "(- 1)", -1),
    ],
)
def test_parse(program: str, expected_parsed_expr: str, expected_evaluated_value: int):
    assert str(tdop.parse(program)) == expected_parsed_expr
    assert tdop.parse(program).evalulate() == expected_evaluated_value
