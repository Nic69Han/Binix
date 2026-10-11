import pytest

from binix.core import detect, entropy, hexdump, strings


@pytest.mark.parametrize("data,expected", [
    (b"\x7fELF\x02\x01", "ELF executable"),
    (b"\x89PNG\r\n\x1a\n....", "PNG image"),
    (b"%PDF-1.7", "PDF document"),
    (b"PK\x03\x04rest", "ZIP archive"),
    (b"\x00" * 257 + b"ustar\x0000", "POSIX tar archive"),
    (b"hello", None),
    (b"", None),
])
def test_detect(data, expected):
    assert detect(data) == expected


def test_entropy_bounds():
    assert entropy(b"") == 0.0
    assert entropy(b"aaaa") == 0.0
    assert entropy(bytes(range(256))) == pytest.approx(8.0)
    assert entropy(b"ab" * 10) == pytest.approx(1.0)


def test_strings():
    data = b"\x00\x01hello\x00ab\x00world!\xff"
    assert list(strings(data)) == [(2, "hello"), (11, "world!")]
    assert list(strings(data, min_len=2)) == [(2, "hello"), (8, "ab"), (11, "world!")]


def test_strings_at_end_of_data():
    assert list(strings(b"\x00tail")) == [(1, "tail")]


def test_strings_rejects_bad_min_len():
    with pytest.raises(ValueError):
        list(strings(b"abc", min_len=0))


def test_hexdump_layout():
    lines = list(hexdump(b"ABC\x00", base=0x10, width=4))
    assert lines == ["00000010  41 42 43 00  |ABC.|"]


def test_hexdump_pads_last_line():
    lines = list(hexdump(b"ABCDE", width=4))
    assert lines[1] == "00000004  45           |E|"
