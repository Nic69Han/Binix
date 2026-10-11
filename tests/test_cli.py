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
