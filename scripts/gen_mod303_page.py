#!/usr/bin/env python3
"""One-off generator: reads DR303 XLSX and prints `field` blocks for a page sheet.

Usage:
    python3 scripts/gen_mod303_page.py DP30301 [path/to/DR303.xlsx]

Emits field blocks (NOT a full record) to stdout. Target ~80% correctness;
the output is meant to be hand-edited.
"""
import re
import sys
import unicodedata
from pathlib import Path

import openpyxl

XLSX_DEFAULT = "/tmp/m303/DR303.xlsx"

ACCENTS = str.maketrans({
    "á": "a", "é": "e", "í": "i", "ó": "o", "ú": "u", "ü": "u",
    "Á": "a", "É": "e", "Í": "i", "Ó": "o", "Ú": "u", "Ü": "u",
    "ñ": "n", "Ñ": "n", "ç": "c", "Ç": "c",
})

QUOTED_RE = re.compile(r'^\s*Constante\s*"([^"]+)"\s*$')
QUOTED_RE2 = re.compile(r'^\s*"([^"]+)"\s*$')
DECIMALS_RE = re.compile(r'(\d+)\s*enteros?\s*y\s*(\d+)\s*decimales?', re.IGNORECASE)
CASILLA_RE = re.compile(r'\[(\d{2,3})\]')


def slugify(text: str) -> str:
    text = (text or "").strip()
    text = text.translate(ACCENTS)
    text = unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode("ascii")
    text = text.lower()
    text = re.sub(r'[^a-z0-9]+', '_', text)
    return text.strip('_')


def map_type(tipo: str, contenido: str | None):
    """Return (type_name, decimals_or_None)."""
    t = (tipo or "").strip()
    c = (contenido or "").strip() if contenido else ""
    if t in ("An", "AN"):
        return ("alphanumeric", None)
    if t == "A":
        return ("alpha", None)
    if t == "Num":
        m = DECIMALS_RE.search(c)
        if m:
            return ("unsigned_amount", int(m.group(2)))
        return ("number", None)
    if t == "N":
        m = DECIMALS_RE.search(c)
        if m:
            return ("signed_amount", int(m.group(2)))
        return ("number", None)
    return ("alphanumeric", None)


def extract_fixed(contenido: str | None):
    if not contenido:
        return None
    s = contenido.strip()
    m = QUOTED_RE.match(s)
    if m:
        return m.group(1)
    m = QUOTED_RE2.match(s)
    if m:
        return m.group(1)
    return None


def build_name(descripcion: str) -> str:
    desc = descripcion or ""
    m = CASILLA_RE.search(desc)
    base = slugify(desc)
    if m:
        casilla = m.group(1).zfill(2)
        # strip the bracket digits from base
        base_no = re.sub(r'_\d{2,3}_?$', '', base)
        return f"c{casilla}_{base_no}" if base_no else f"c{casilla}"
    return base


def main():
    if len(sys.argv) < 2:
        print("usage: gen_mod303_page.py <SHEET> [xlsx]", file=sys.stderr)
        sys.exit(2)
    sheet = sys.argv[1]
    xlsx = Path(sys.argv[2]) if len(sys.argv) > 2 else Path(XLSX_DEFAULT)
    wb = openpyxl.load_workbook(xlsx, data_only=True)
    ws = wb[sheet]

    for row in ws.iter_rows(min_row=2, values_only=True):
        if row[0] is None or not isinstance(row[0], int):
            continue
        n, posic, lon, tipo, desc, valid, contenido = row[:7]
        if posic is None or lon is None:
            continue
        type_name, decimals = map_type(tipo, contenido)
        fixed = extract_fixed(contenido)
        name = build_name(desc)
        print(f"    # {desc}")
        parts = [
            f"at = {posic}",
            f"length = {lon}",
            f"type = {type_name}",
        ]
        if fixed is not None:
            parts.append(f'fixed = "{fixed}"')
        if decimals is not None:
            parts.append(f"decimals = {decimals}")
        print(f"    field {name} {{")
        for p in parts:
            print(f"        {p}")
        print("    }")
        print()


if __name__ == "__main__":
    main()
