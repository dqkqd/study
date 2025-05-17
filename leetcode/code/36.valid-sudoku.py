from collections import defaultdict


def valid(a: list[str]) -> bool:
    count: defaultdict[str, int] = defaultdict(int)
    for x in a:
        if x != ".":
            count[x] += 1
        if count[x] > 1:
            return False
    return True


class Solution:
    def isValidSudoku(self, board: list[list[str]]) -> bool:
        for row in board:
            if not valid(row):
                return False
        for c in range(0, 9):
            col = [board[i][c] for i in range(0, 9)]
            if not valid(col):
                return False
        for sx in range(0, 9, 3):
            for sy in range(0, 9, 3):
                square: list[str] = []
                for dx in range(3):
                    for dy in range(3):
                        square.append(board[sx + dx][sy + dy])
                if not valid(square):
                    return False

        return True
