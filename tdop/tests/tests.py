import pytest
from inline_snapshot import snapshot

import tdop


@pytest.mark.parametrize(
    ["program", "expected_parsed_expr", "expected_evaluated_value"],
    [
        ("1", snapshot("1"), snapshot(1)),
        ("+1", snapshot("1"), snapshot(1)),
        ("-1", snapshot("(- 1)"), snapshot(-1)),
        ("1 + 2", snapshot("(+ 1 2)"), snapshot(3)),
        ("1 - 2", snapshot("(- 1 2)"), snapshot(-1)),
        ("2 * 3", snapshot("(* 2 3)"), snapshot(6)),
        ("3 / 4", snapshot("(/ 3 4)"), snapshot(0.75)),
        ("1 + 3 + 2", snapshot("(+ (+ 1 3) 2)"), snapshot(6)),
        ("1 + 3 * 2", snapshot("(+ 1 (* 3 2))"), snapshot(7)),
        ("1 + 3 - 2", snapshot("(- (+ 1 3) 2)"), snapshot(2)),
        ("1 + 3 / 2", snapshot("(+ 1 (/ 3 2))"), snapshot(2.5)),
        ("1 + 2 * 3 * 4 + 5", snapshot("(+ (+ 1 (* (* 2 3) 4)) 5)"), snapshot(30)),
        ("1 + 2 * 3 * 4 * 5", snapshot("(+ 1 (* (* (* 2 3) 4) 5))"), snapshot(121)),
        ("(1)", snapshot("1"), snapshot(1)),
        ("(-1)", snapshot("(- 1)"), snapshot(-1)),
        ("(+1)", snapshot("1"), snapshot(1)),
        ("-(+1)", snapshot("(- 1)"), snapshot(-1)),
        ("-(2 + 3)", snapshot("(- (+ 2 3))"), snapshot(-5)),
        ("2 * (3 + 4)", snapshot("(* 2 (+ 3 4))"), snapshot(14)),
        ("(3 + 4) * 2", snapshot("(* (+ 3 4) 2)"), snapshot(14)),
    ],
)
def test_parse(
    program: str, expected_parsed_expr: str, expected_evaluated_value: float
):
    assert str(tdop.parse(program)) == expected_parsed_expr
    assert tdop.parse(program).evalulate() == expected_evaluated_value
