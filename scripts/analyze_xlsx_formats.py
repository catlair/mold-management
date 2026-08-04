import json
import re
import sys
import unicodedata
import zipfile
from collections import Counter, defaultdict
from pathlib import PurePosixPath
import xml.etree.ElementTree as ET

MAIN_NS = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
REL_NS = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PKG_REL_NS = "http://schemas.openxmlformats.org/package/2006/relationships"
NS = {"m": MAIN_NS, "r": REL_NS, "p": PKG_REL_NS}

HEADER_KEYWORDS = (
    "规格",
    "尺寸",
    "直径",
    "长度",
    "高度",
    "杆长",
    "线径",
    "孔径",
    "范围",
    "公差",
    "厚度",
    "宽度",
    "外径",
    "内径",
    "diameter",
    "length",
    "size",
    "spec",
)

VARIANT_PATTERNS = {
    "asterisk": re.compile(r"\*"),
    "lower_x": re.compile(r"x"),
    "upper_x": re.compile(r"X"),
    "multiply": re.compile(r"×"),
    "ascii_tilde": re.compile(r"~"),
    "fullwidth_tilde": re.compile(r"[～〜]"),
    "ascii_hyphen": re.compile(r"-"),
    "unicode_minus": re.compile(r"−"),
    "en_em_dash": re.compile(r"[–—]"),
    "plus_minus": re.compile(r"±"),
    "fullwidth_plus_minus": re.compile(r"＋|－"),
    "space": re.compile(r"\s"),
    "fullwidth_char": re.compile(r"[！-～]"),
}


def column_number(cell_ref: str) -> int:
    letters = "".join(character for character in cell_ref if character.isalpha())
    result = 0
    for character in letters:
        result = result * 26 + ord(character.upper()) - 64
    return result


def column_label(index: int) -> str:
    result = ""
    while index:
        index, remainder = divmod(index - 1, 26)
        result = chr(65 + remainder) + result
    return result


def read_shared_strings(archive: zipfile.ZipFile) -> list[str]:
    if "xl/sharedStrings.xml" not in archive.namelist():
        return []
    root = ET.fromstring(archive.read("xl/sharedStrings.xml"))
    return [
        "".join(node.text or "" for node in item.iter(f"{{{MAIN_NS}}}t"))
        for item in root.findall("m:si", NS)
    ]


def resolve_sheet_paths(archive: zipfile.ZipFile) -> list[tuple[str, str]]:
    workbook = ET.fromstring(archive.read("xl/workbook.xml"))
    relationships = ET.fromstring(archive.read("xl/_rels/workbook.xml.rels"))
    targets = {
        relationship.attrib["Id"]: relationship.attrib["Target"]
        for relationship in relationships.findall("p:Relationship", NS)
    }
    result = []
    for sheet in workbook.find("m:sheets", NS):
        relationship_id = sheet.attrib[f"{{{REL_NS}}}id"]
        target = targets[relationship_id].replace("\\", "/")
        if target.startswith("/"):
            path = target.lstrip("/")
        else:
            path = str(PurePosixPath("xl") / target)
        while "/../" in path:
            parts = []
            for part in path.split("/"):
                if part == "..":
                    if parts:
                        parts.pop()
                elif part != ".":
                    parts.append(part)
            path = "/".join(parts)
        result.append((sheet.attrib["name"], path))
    return result


def read_cell(cell: ET.Element, shared_strings: list[str]) -> str:
    cell_type = cell.attrib.get("t")
    if cell_type == "inlineStr":
        inline_string = cell.find("m:is", NS)
        if inline_string is None:
            return ""
        return "".join(
            node.text or "" for node in inline_string.iter(f"{{{MAIN_NS}}}t")
        )
    value = cell.find("m:v", NS)
    if value is None or value.text is None:
        return ""
    if cell_type == "s":
        return shared_strings[int(value.text)]
    if cell_type == "b":
        return "TRUE" if value.text == "1" else "FALSE"
    return value.text


def read_rows(
    archive: zipfile.ZipFile, sheet_path: str, shared_strings: list[str]
) -> list[dict[int, str]]:
    root = ET.fromstring(archive.read(sheet_path))
    rows = []
    for row in root.findall(".//m:sheetData/m:row", NS):
        values = {}
        for cell in row.findall("m:c", NS):
            text = read_cell(cell, shared_strings)
            if text != "":
                values[column_number(cell.attrib.get("r", ""))] = text
        rows.append(values)
    return rows


def normalized_dimension_key(value: str) -> str:
    text = unicodedata.normalize("NFKC", value).strip()
    text = re.sub(r"\s+", "", text)
    text = re.sub(r"(?<=\d)[*xX×](?=\d)", "×", text)
    text = re.sub(r"[～〜]", "~", text)
    text = text.replace("−", "-").replace("–", "-").replace("—", "-")
    text = re.sub(r"(?<=\d)\.0+(?=($|[^\d]))", "", text)
    return text.casefold()


def has_relevant_symbol(value: str) -> bool:
    return bool(re.search(r"[*xX×~～〜±＋－−–—]", value))


def summarize_column(values: list[str]) -> dict:
    stripped_values = [str(value).strip() for value in values if str(value).strip()]
    counts = Counter(stripped_values)
    variants = {
        name: sum(1 for value in stripped_values if pattern.search(value))
        for name, pattern in VARIANT_PATTERNS.items()
    }
    variants = {name: count for name, count in variants.items() if count}

    normalized_groups = defaultdict(list)
    for value in counts:
        normalized_groups[normalized_dimension_key(value)].append(value)
    collisions = []
    for key, originals in normalized_groups.items():
        if len(originals) > 1:
            collisions.append(
                {
                    "normalized": key,
                    "originals": sorted(
                        (
                            {"value": original, "count": counts[original]}
                            for original in originals
                        ),
                        key=lambda item: (-item["count"], item["value"]),
                    ),
                }
            )
    collisions.sort(
        key=lambda item: -sum(original["count"] for original in item["originals"])
    )

    symbol_examples = [
        {"value": value, "count": count}
        for value, count in counts.most_common()
        if has_relevant_symbol(value)
    ][:40]

    return {
        "count": len(stripped_values),
        "uniqueCount": len(counts),
        "variants": variants,
        "topValues": [
            {"value": value, "count": count} for value, count in counts.most_common(30)
        ],
        "symbolExamples": symbol_examples,
        "normalizationCollisions": collisions[:30],
    }


def analyze(workbook_path: str) -> dict:
    result = {"workbook": workbook_path, "sheets": [], "globalSymbolExamples": []}
    global_symbol_values = Counter()

    with zipfile.ZipFile(workbook_path) as archive:
        shared_strings = read_shared_strings(archive)
        for sheet_name, sheet_path in resolve_sheet_paths(archive):
            rows = read_rows(archive, sheet_path, shared_strings)
            if not rows:
                result["sheets"].append(
                    {"name": sheet_name, "rowCount": 0, "headerRow": None, "columns": []}
                )
                continue

            header_row_index = 0
            best_score = -1
            for index, row in enumerate(rows[:15]):
                score = sum(
                    1
                    for value in row.values()
                    if any(keyword in str(value).lower() for keyword in HEADER_KEYWORDS)
                )
                if score > best_score:
                    best_score = score
                    header_row_index = index

            headers = rows[header_row_index]
            data_rows = rows[header_row_index + 1 :]
            candidate_columns = []
            for index, header in sorted(headers.items()):
                header_text = str(header).strip()
                values = [row.get(index, "") for row in data_rows]
                relevant_header = any(
                    keyword in header_text.lower() for keyword in HEADER_KEYWORDS
                )
                relevant_values = sum(
                    1 for value in values if has_relevant_symbol(str(value))
                )
                if relevant_header or relevant_values:
                    summary = summarize_column(values)
                    summary.update(
                        {
                            "column": column_label(index),
                            "header": header_text,
                            "selectedBy": {
                                "headerKeyword": relevant_header,
                                "symbolValueCount": relevant_values,
                            },
                        }
                    )
                    candidate_columns.append(summary)
                    for value in values:
                        value_text = str(value).strip()
                        if value_text and has_relevant_symbol(value_text):
                            global_symbol_values[value_text] += 1

            result["sheets"].append(
                {
                    "name": sheet_name,
                    "rowCount": len(data_rows),
                    "headerRow": header_row_index + 1,
                    "headers": {
                        column_label(index): str(value).strip()
                        for index, value in sorted(headers.items())
                    },
                    "columns": candidate_columns,
                }
            )

    result["globalSymbolExamples"] = [
        {"value": value, "count": count}
        for value, count in global_symbol_values.most_common(100)
    ]
    return result


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: analyze_xlsx_formats.py INPUT.xlsx OUTPUT.json")
    analysis = analyze(sys.argv[1])
    with open(sys.argv[2], "w", encoding="utf-8") as output_file:
        json.dump(analysis, output_file, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
