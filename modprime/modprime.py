#!/usr/bin/env python3

import argparse
import random
import sys


p = 2**89 - 1
m = 2**20

def mod_prime_gen_params():
    a = random.randrange(1, p)
    b = random.randrange(0, p)
    return (a, b)

def mod_prime(params, x):
    a, b = params
    return ((a*x + b) % p) % m


l = 20

def shift_gen_params():
    a = 2 * random.randrange(1, p//2)
    return (a,)

def shift(params, x):
    a, = params
    return a*x >> (64 - l)


# (num_params, gen_params, func)
func_mod_prime = (2, mod_prime_gen_params, mod_prime)
func_shift = (1, shift_gen_params, shift)


def do_generate(parser, func, args):
    num_params, gen_params, func = func

    if len(args.args) < 1:
        parser.error("missing required argument: count")
    elif len(args.args) > 1:
        parser.error("unexpected extra arguments")
    try:
        count = int(args.args[0])
    except ValueError:
        parser.error("count argument should be an integer")
    if count <= 0:
        parser.error("count argument should be positive")

    for i in range(count):
        a = random.randrange(1, p)
        b = random.randrange(0, p)
        x = random.randrange(0, 2**64)
        y = mod_prime(a, b, x)

        print(f"0x{a:023x},0x{b:023x},0x{x:016x},0x{y:05x}")


def parse_int(s):
    if s.startswith("0x"):
        return int(s[2:], 16)
    else:
        return int(s)


def do_read_csv(parser, args):
    if len(args.args) > 0:
        parser.error("unexpected extra arguments")

    for line in sys.stdin:
        str_a, str_b, str_x = line.split(",")

        a = parse_int(str_a)
        b = parse_int(str_b)
        x = parse_int(str_x)
        y = mod_prime(a, b, x)

        print(f"0x{y:05x}")


def do_check(parser, args):
    if len(args.args) > 0:
        parser.error("unexpected extra arguments")

    num_ok = 0
    num_err = 0

    dot = False
    for line in sys.stdin:
        str_a, str_b, str_x, str_y = line.split(",")

        a = parse_int(str_a)
        b = parse_int(str_b)
        x = parse_int(str_x)
        actual = parse_int(str_y)
        expected = mod_prime(a, b, x)

        if actual == expected:
            num_ok += 1
            print(".", end="")
            dot = True
            sys.stdout.flush()
        else:
            num_err += 1
            if dot:
                print()
                dot = False
            print(f"mismatch: 0x{a:023x},0x{b:023x},0x{x:016x} expected: 0x{expected:05x}, actual: 0x{actual:05x}")
    if dot:
        print()
        dot = False

    num_total = num_ok + num_err

    if num_total == 0:
        print("done (no input)")
    else:
        print()
        print(f"done (correct: {num_ok}, incorrect: {num_err}, error rate: {num_err/num_total:.2%})")


def main():
    parser = argparse.ArgumentParser()

    mode_group = parser.add_mutually_exclusive_group(required=True)
    mode_group.add_argument("-g", "--generate",
                            dest="mode", action="store_const", const=do_generate,
                            help="generate random samples, count given as argument")
    mode_group.add_argument("-r", "--read-csv",
                            dest="mode", action="store_const", const=do_read_csv,
                            help="read comma-separated a, b and x values from standard input")
    mode_group.add_argument("-c", "--check",
                            dest="mode", action="store_const", const=do_check,
                            help="read comma-separated a, b, x and y, and check whether y is correct")

    parser.add_argument("args", nargs="...", metavar="...", help="positional arguments depend on mode")

    args = parser.parse_args()

    args.mode(parser, args)


if __name__ == '__main__':
    main()
