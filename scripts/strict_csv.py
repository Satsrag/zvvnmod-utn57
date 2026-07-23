"""Fail-closed CSV parsing shared by mapping generators and verification."""

from __future__ import annotations

import json
from typing import Any


def _rows(text: str) -> list[list[str]]:
    source = text.replace("\r\n", "\n").replace("\r", "\n")
    rows: list[list[str]] = []
    row: list[str] = []
    field: list[str] = []
    quoted = False
    quote_closed = False
    index = 0
    while index < len(source):
        character = source[index]
        if quoted:
            if character == '"' and index + 1 < len(source) and source[index + 1] == '"':
                field.append('"')
                index += 1
            elif character == '"':
                quoted = False
                quote_closed = True
            else:
                field.append(character)
        elif quote_closed:
            if character == ",":
                row.append("".join(field))
                field = []
                quote_closed = False
            elif character == "\n":
                row.append("".join(field))
                rows.append(row)
                row = []
                field = []
                quote_closed = False
            else:
                raise ValueError("unexpected character after quoted CSV field")
        elif character == '"' and not field:
            quoted = True
        elif character == '"':
            raise ValueError("quote in unquoted CSV field")
        elif character == ",":
            row.append("".join(field))
            field = []
        elif character == "\n":
            row.append("".join(field))
            rows.append(row)
            row = []
            field = []
        else:
            field.append(character)
        index += 1
    if quoted:
        raise ValueError("unterminated quoted CSV field")
    if field or row or quote_closed:
        row.append("".join(field))
        rows.append(row)
    while rows and all(value == "" for value in rows[-1]):
        rows.pop()
    if not rows:
        raise ValueError("CSV must contain a header")
    return rows


def parse_table(text: str, expected_headers: list[str]) -> list[dict[str, str]]:
    rows = _rows(text)
    headers = rows.pop(0)
    if headers != expected_headers:
        raise ValueError("CSV headers differ from schema")
    if len(set(headers)) != len(headers):
        raise ValueError("CSV headers must be unique")
    result: list[dict[str, str]] = []
    for index, values in enumerate(rows):
        if len(values) != len(headers):
            raise ValueError(f"CSV row {index} has the wrong width")
        result.append(dict(zip(headers, values, strict=True)))
    return result


def parse_metadata_table(
    text: str,
    expected_headers: list[str],
    expected_metadata_keys: list[str],
) -> tuple[dict[str, Any], list[dict[str, str]]]:
    normalized = text.replace("\r\n", "\n").replace("\r", "\n")
    first_line, separator, csv_text = normalized.partition("\n")
    if not separator or not first_line.startswith("# metadata="):
        raise ValueError("CSV metadata missing")
    metadata_text = first_line.removeprefix("# metadata=")
    try:
        metadata = json.loads(metadata_text)
    except json.JSONDecodeError as error:
        raise ValueError("CSV metadata is not valid JSON") from error
    canonical = json.dumps(metadata, ensure_ascii=False, separators=(",", ":"))
    if canonical != metadata_text:
        raise ValueError("CSV metadata must use canonical JSON without duplicate keys")
    if not isinstance(metadata, dict) or list(metadata) != expected_metadata_keys:
        raise ValueError("CSV metadata fields differ from schema")
    return metadata, parse_table(csv_text, expected_headers)
