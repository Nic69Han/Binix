"""Pure functions behind the binix commands."""

import math
from collections import Counter
from typing import Iterator, List, Optional, Tuple

# (offset, signature, description). Checked in order; first match wins.
MAGIC: List[Tuple[int, bytes, str]] = [
    (0, b"\x7fELF", "ELF executable"),
    (0, b"MZ", "DOS/Windows executable (PE)"),
    (0, b"\xcf\xfa\xed\xfe", "Mach-O 64-bit executable"),
    (0, b"\xce\xfa\xed\xfe", "Mach-O 32-bit executable"),
    (0, b"\xca\xfe\xba\xbe", "Mach-O universal binary or Java class"),
    (0, b"\x89PNG\r\n\x1a\n", "PNG image"),
    (0, b"\xff\xd8\xff", "JPEG image"),
    (0, b"GIF87a", "GIF image"),
    (0, b"GIF89a", "GIF image"),
    (0, b"%PDF-", "PDF document"),
    (0, b"PK\x03\x04", "ZIP archive"),
    (0, b"\x1f\x8b", "gzip compressed data"),
    (0, b"BZh", "bzip2 compressed data"),
    (0, b"\xfd7zXZ\x00", "xz compressed data"),
    (0, b"7z\xbc\xaf\x27\x1c", "7-Zip archive"),
    (0, b"\x28\xb5\x2f\xfd", "Zstandard compressed data"),
    (0, b"\x00asm", "WebAssembly binary"),
    (0, b"SQLite format 3\x00", "SQLite 3 database"),
    (257, b"ustar", "POSIX tar archive"),
]


def detect(data: bytes) -> Optional[str]:
    """Return a description of the file type, or None if unknown."""
    for offset, sig, desc in MAGIC:
        if data[offset:offset + len(sig)] == sig:
            return desc
    return None


def entropy(data: bytes) -> float:
    """Shannon entropy in bits per byte (0.0 to 8.0)."""
    if not data:
        return 0.0
    n = len(data)
    return -sum(c / n * math.log2(c / n) for c in Counter(data).values())


def strings(data: bytes, min_len: int = 4) -> Iterator[Tuple[int, str]]:
    """Yield (offset, text) for runs of printable ASCII of at least min_len."""
    if min_len < 1:
        raise ValueError("min_len must be >= 1")
    start = None
    for i, b in enumerate(data):
        if 0x20 <= b < 0x7f or b == 0x09:
            if start is None:
                start = i
        else:
            if start is not None and i - start >= min_len:
                yield start, data[start:i].decode("ascii")
            start = None
    if start is not None and len(data) - start >= min_len:
        yield start, data[start:].decode("ascii")


def hexdump(data: bytes, base: int = 0, width: int = 16) -> Iterator[str]:
    """Yield lines in the classic `offset  hex bytes  |ascii|` layout."""
    if width < 1:
        raise ValueError("width must be >= 1")
    for i in range(0, len(data), width):
        chunk = data[i:i + width]
        hexpart = " ".join(f"{b:02x}" for b in chunk).ljust(width * 3 - 1)
        text = "".join(chr(b) if 0x20 <= b < 0x7f else "." for b in chunk)
        yield f"{base + i:08x}  {hexpart}  |{text}|"
