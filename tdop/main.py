import sys
import tdop


def main(program: str):
    expr = tdop.parse(program)
    print(expr.evalulate())


if __name__ == "__main__":
    main(sys.argv[1])
