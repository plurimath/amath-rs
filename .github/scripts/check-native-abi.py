"""Exercise the stage ABI using the library built by the platform's Cargo job."""
import ctypes
import json
from pathlib import Path
import sys


def main():
    stem = sys.argv[1]
    if sys.platform == "win32":
        filename = f"{stem}.dll"
    elif sys.platform == "darwin":
        filename = f"lib{stem}.dylib"
    else:
        filename = f"lib{stem}.so"
    library = ctypes.CDLL(str((Path("target/debug") / filename).resolve()))
    version = getattr(library, f"{stem}_abi_version")
    version.argtypes = []
    version.restype = ctypes.c_uint32
    assert version() == 1
    parse = getattr(library, f"{stem}_parse_model_json_v1")
    parse.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t]
    parse.restype = ctypes.c_void_p
    free = getattr(library, f"{stem}_free_json_v1")
    free.argtypes = [ctypes.c_void_p]
    free.restype = None
    cases = [(b"x+1", b".", True), (b"1,5", b",", True),
             (b"x", b"invalid", False), (b"\xff", b".", False)]
    for text, decimal, expected in cases:
        pointer = parse(text, len(text), decimal, len(decimal))
        assert pointer, "native ABI returned a null result"
        try:
            reply = json.loads(ctypes.string_at(pointer).decode("utf-8"))
        finally:
            free(pointer)
        assert reply["abi_version"] == 1, reply
        assert reply["ok"] is expected, reply
        if expected:
            assert reply["model"]["decimal"] == decimal.decode("ascii"), reply
        else:
            assert reply["error"]["category"] == "invalid_argument", reply
    free(None)
    print(f"{stem}: four native ABI cases passed")


if __name__ == "__main__":
    main()
