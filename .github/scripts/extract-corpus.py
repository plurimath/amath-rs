"""Verify a pinned plurimath-testsuite unicode generation and extract its rows.

Usage: extract-corpus.py <testsuite-root> <pin.json> <out.json>

Every check is against the pin: the recomputed snapshot digest, the
ledger entry, the files on disk versus provenance, provenance SHA-256, each payload's byte size
and SHA-256 as recorded in provenance, and the expected row IDs of each pinned
boundary (count and SHA-256 of the newline-joined IDs in payload order). Rows
are copied without changing their expected values.
"""
import hashlib
import json
from pathlib import Path
import sys

import yaml

Loader = getattr(yaml, "CSafeLoader", yaml.SafeLoader)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def snapshot_digest(directory):
    """sha256 over '<sha256>  <relative path>\\n' lines ordered by path bytes."""
    files = {}
    for path in sorted(directory.rglob("*")):
        assert not path.is_symlink(), f"symlink in generation: {path}"
        if path.is_file():
            files[path.relative_to(directory).as_posix()] = path.read_bytes()
    assert files, "empty generation"
    lines = "".join(
        f"{sha256(files[name])}  {name}\n" for name in sorted(files, key=lambda n: n.encode("utf-8"))
    )
    return "sha256:" + sha256(lines.encode("utf-8")), set(files)


def main():
    root, pin_path, out_path = (Path(arg) for arg in sys.argv[1:4])
    pin = json.loads(pin_path.read_text(encoding="utf-8"))
    generation = (root / pin["path"]).resolve()
    digest, on_disk = snapshot_digest(generation)
    assert digest == pin["snapshot_digest"], f"snapshot digest {digest} differs from the pin"
    ledger = yaml.load((root / "generations" / f"{pin['language']}.yaml").read_bytes(), Loader=Loader)
    entries = [e for e in ledger["entries"] if e.get("generation") == pin["generation"]]
    assert len(entries) == 1, "ledger entry for the pinned generation"
    assert entries[0]["snapshot_digest"] == pin["snapshot_digest"], "ledger digest differs from the pin"
    assert entries[0]["path"] == pin["path"], "ledger path differs from the pin"
    provenance_bytes = (generation / "provenance.yaml").read_bytes()
    assert sha256(provenance_bytes) == pin["provenance_sha256"], "provenance changed"
    provenance = yaml.load(provenance_bytes, Loader=Loader)
    assert provenance["language"] == pin["language"], "language"
    assert int(provenance["generation"]) == pin["generation"], "generation"
    listed = {payload["path"] for payload in provenance["payloads"]}
    assert listed == on_disk - {"provenance.yaml"}, "payload files differ from provenance"
    boundaries = pin["boundaries"]
    rows = []
    for payload in provenance["payloads"]:
        path = (generation / payload["path"]).resolve()
        assert generation in path.parents, f"{payload['path']} escapes generation"
        data = path.read_bytes()
        assert len(data) == payload["bytes"], f"{payload['path']} size"
        assert sha256(data) == payload["sha256"], f"{payload['path']} checksum"
        document = yaml.load(data, Loader=Loader)
        if not isinstance(document, dict) or document.get("boundary") not in boundaries:
            continue
        for case in document["cases"]:
            case["payload"] = payload["path"]
            case["boundary"] = document["boundary"]
            rows.append(case)
    assert len({row["id"] for row in rows}) == len(rows), "duplicate row id"
    summary = {}
    for boundary, expected in boundaries.items():
        ids = [row["id"] for row in rows if row["boundary"] == boundary]
        digest = sha256("\n".join(ids).encode("utf-8"))
        assert len(ids) == expected["rows"], f"{boundary}: {len(ids)} rows"
        assert digest == expected["ids_sha256"], f"{boundary}: row ID digest {digest}"
        summary[boundary] = len(ids)
    out = {key: pin[key] for key in ("language", "generation", "revision", "path")}
    out["cases"] = rows
    Path(out_path).write_text(json.dumps(out, ensure_ascii=False), encoding="utf-8")
    print(json.dumps({"generation": pin["generation"], "rows": summary}))


if __name__ == "__main__":
    main()
