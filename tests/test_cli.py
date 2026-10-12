from binix.cli import main


def test_info(tmp_path, capsys):
    f = tmp_path / "a.bin"
    f.write_bytes(b"\x7fELF" + b"\x00" * 12)
    assert main(["info", str(f)]) == 0
    out = capsys.readouterr().out
    assert "size:    16 bytes" in out
    assert "ELF executable" in out


def test_hex_offset_and_length(tmp_path, capsys):
    f = tmp_path / "a.bin"
    f.write_bytes(b"0123456789")
    assert main(["hex", str(f), "-s", "2", "-n", "3"]) == 0
    assert capsys.readouterr().out.startswith("00000002  32 33 34")


def test_strings_with_offsets(tmp_path, capsys):
    f = tmp_path / "a.bin"
    f.write_bytes(b"\x00\x00binix\x00")
    assert main(["strings", "-t", str(f)]) == 0
    assert capsys.readouterr().out == "00000002 binix\n"


def test_missing_file(capsys):
    assert main(["info", "/nonexistent/file"]) == 1
    assert "binix:" in capsys.readouterr().err


def test_diff(tmp_path, capsys):
    a, b = tmp_path / "a", tmp_path / "b"
    a.write_bytes(b"hello world")
    b.write_bytes(b"hellO world!")
    assert main(["diff", "-c", "4", str(a), str(b)]) == 1
    out = capsys.readouterr().out
    assert "00000004  1 byte(s) differ" in out
    assert "  - 6f\n  + 4f" in out
    assert "0000000b  1 byte(s) differ" in out and "(eof)" in out
    assert "sizes differ: 11 vs 12 bytes" in out


def test_diff_identical(tmp_path, capsys):
    a = tmp_path / "a"
    a.write_bytes(b"same")
    assert main(["diff", str(a), str(a)]) == 0
    assert capsys.readouterr().out == ""
