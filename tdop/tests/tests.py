import pytest

import tdop


@pytest.mark.parametrize(
    ["program", "expected_parsed_expr", "expected_evaluated_value"],
    [
        ("1", "1", 1),
        ("+1", "(+ 1)", 1),
        ("-1", "(- 1)", -1),
        ("1 + 2", "(+ 1 2)", 3),
        ("1 - 2", "(- 1 2)", -1),
        ("2 * 3", "(* 2 3)", 6),
        ("3 / 4", "(/ 3 4)", 0.75),
    ],
)
def test_parse(
    program: str, expected_parsed_expr: str, expected_evaluated_value: float
):
    assert str(tdop.parse(program)) == expected_parsed_expr
    assert tdop.parse(program).evalulate() == expected_evaluated_value
