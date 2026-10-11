"""Command-line entry point: `binix <command> FILE`."""

import argparse
import sys
from typing import List, Optional

from . import __version__
from .core import detect, entropy, hexdump, strings
from .formats import FormatError, parse


def _read(path: str, offset: int = 0, length: Optional[int] = None) -> bytes:
    with open(path, "rb") as f:
        f.seek(offset)
        return f.read() if length is None else f.read(length)


def _non_negative(value: str) -> int:
    n = int(value, 0)
    if n < 0:
        raise argparse.ArgumentTypeError("must be >= 0")
    return n


def _positive(value: str) -> int:
    n = int(value, 0)
    if n < 1:
        raise argparse.ArgumentTypeError("must be >= 1")
    return n


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="binix", description="Inspect binary files.")
    p.add_argument("--version", action="version", version=f"binix {__version__}")
    sub = p.add_subparsers(dest="command", required=True)

    info = sub.add_parser("info", help="size, detected type and entropy")
    info.add_argument("file")

    hx = sub.add_parser("hex", help="hexdump")
    hx.add_argument("file")
    hx.add_argument("-s", "--offset", type=_non_negative, default=0)
    hx.add_argument("-n", "--length", type=_non_negative)
    hx.add_argument("-w", "--width", type=_positive, default=16)

    st = sub.add_parser("strings", help="printable ASCII strings")
    st.add_argument("file")
    st.add_argument("-m", "--min-len", type=_positive, default=4)
    st.add_argument("-t", "--offsets", action="store_true", help="print hex offsets")

    hd = sub.add_parser("headers", help="ELF/PE header fields and sections")
    hd.add_argument("file")

    return p


def _print_headers(path: str) -> int:
    h = parse(_read(path))
    if h is None:
        print(f"binix: {path}: not an ELF or PE file", file=sys.stderr)
        return 1
    width = max(len(k) for k in h.fields)
    for k, v in h.fields.items():
        print(f"{k + ':':<{width + 1}} {v}")
    if h.sections:
        print()
        print(f"{'idx':>3}  {'name':<20} {'offset':>10} {'size':>10} {'address':>18}")
        for i, s in enumerate(h.sections):
            print(f"{i:>3}  {s.name[:20]:<20} {s.offset:>#10x} {s.size:>#10x} {s.address:>#18x}")
    return 0


def main(argv: Optional[List[str]] = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        if args.command == "info":
            data = _read(args.file)
            print(f"file:    {args.file}")
            print(f"size:    {len(data)} bytes")
            print(f"type:    {detect(data) or 'unknown'}")
            print(f"entropy: {entropy(data):.4f} bits/byte")
        elif args.command == "hex":
            data = _read(args.file, args.offset, args.length)
            for line in hexdump(data, base=args.offset, width=args.width):
                print(line)
        elif args.command == "strings":
            for off, text in strings(_read(args.file), args.min_len):
                print(f"{off:08x} {text}" if args.offsets else text)
        elif args.command == "headers":
            return _print_headers(args.file)
    except FormatError as e:
        print(f"binix: {args.file}: malformed headers: {e}", file=sys.stderr)
        return 1
    except BrokenPipeError:
        return 0
    except OSError as e:
        print(f"binix: {e.filename}: {e.strerror}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
