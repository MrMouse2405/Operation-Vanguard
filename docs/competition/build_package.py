#!/usr/bin/env python3
"""Build an editable competition package from the checked-in Markdown source.

Requires python-docx. Convert the resulting DOCX with LibreOffice for the PDF.
"""
from pathlib import Path
import re

from docx import Document
from docx.enum.table import WD_TABLE_ALIGNMENT, WD_CELL_VERTICAL_ALIGNMENT
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.oxml import OxmlElement
from docx.oxml.ns import qn
from docx.shared import Inches, Pt, RGBColor

ROOT = Path(__file__).resolve().parent
RED = "B51F2B"
INK = "20252B"


def inline(paragraph, text):
    for token in re.split(r"(\*\*.*?\*\*|`[^`]+`)", text):
        run = paragraph.add_run(token[2:-2] if token.startswith("**") else token[1:-1] if token.startswith("`") else token)
        if token.startswith("**"):
            run.bold = True
        if token.startswith("`"):
            run.font.name = "Liberation Mono"
            run.font.size = Pt(9)


def shade(cell, color):
    element = OxmlElement("w:shd")
    element.set(qn("w:fill"), color)
    cell._tc.get_or_add_tcPr().append(element)


def field(paragraph, name):
    element = OxmlElement("w:fldSimple")
    element.set(qn("w:instr"), name)
    paragraph._p.append(element)


def table(document, rows):
    columns = len(rows[0])
    result = document.add_table(rows=0, cols=columns)
    result.alignment = WD_TABLE_ALIGNMENT.CENTER
    result.autofit = False
    widths = {2: [5.55, .95], 3: [3.6, .8, 2.1], 4: [1.25, .9, 3.25, 1.1], 5: [1.1, 1.35, 1.35, 1.35, 1.35]}[columns]
    if rows[0] == ["Use", "Rule"]:
        widths = [1.6, 4.9]
    if rows[0] == ["Assessed criterion", "Maximum", "Awarded"]:
        widths = [4.6, .95, .95]
    if rows[0] == ["Component", "Points", "Evidence"]:
        widths = [2.0, .55, 3.95]
    for column, width in zip(result.columns, widths):
        column.width = Inches(width)
    for index, values in enumerate(rows):
        row = result.add_row()
        no_split = OxmlElement("w:cantSplit")
        row._tr.get_or_add_trPr().append(no_split)
        if index == 0:
            repeat = OxmlElement("w:tblHeader")
            row._tr.get_or_add_trPr().append(repeat)
        for cell, text, width in zip(row.cells, values, widths):
            cell.width = Inches(width)
            cell.vertical_alignment = WD_CELL_VERTICAL_ALIGNMENT.TOP
            paragraph = cell.paragraphs[0]
            paragraph.paragraph_format.space_before = Pt(3)
            paragraph.paragraph_format.space_after = Pt(3)
            paragraph.paragraph_format.line_spacing = 1.02
            inline(paragraph, text)
            for run in paragraph.runs:
                run.font.size = Pt(9 if columns == 5 else 9.5)
                if index == 0:
                    run.bold = True
                    run.font.color.rgb = RGBColor.from_string("FFFFFF")
            shade(cell, RED if index == 0 else "F4F4F4" if index % 2 else "FFFFFF")
    spacer = document.add_paragraph()
    spacer.paragraph_format.space_after = Pt(0)
    spacer.paragraph_format.line_spacing = Pt(1)
    spacer.add_run().font.size = Pt(1)


def build():
    document = Document()
    section = document.sections[0]
    section.page_width = Inches(8.5)
    section.page_height = Inches(11)
    section.top_margin = section.bottom_margin = Inches(.8)
    section.left_margin = section.right_margin = Inches(1)
    section.header_distance = section.footer_distance = Inches(.35)
    section.different_first_page_header_footer = True
    normal = document.styles["Normal"]
    normal.font.name = "Calibri"
    normal.font.size = Pt(10.5)
    normal.font.color.rgb = RGBColor.from_string(INK)
    normal.paragraph_format.line_spacing = 1.05
    normal.paragraph_format.space_after = Pt(6)
    for name, size in [("Title", 36), ("Subtitle", 20), ("Heading 1", 18), ("Heading 2", 12.5), ("Heading 3", 11)]:
        style = document.styles[name]
        style.font.name = "Calibri"
        style.font.size = Pt(size)
        style.font.color.rgb = RGBColor.from_string(RED)
        style.paragraph_format.space_before = Pt(10)
        style.paragraph_format.space_after = Pt(6)
        style.paragraph_format.keep_with_next = True
    document.styles["Subtitle"].font.italic = False
    for border in document.styles["Title"].element.iter(qn("w:bottom")):
        border.set(qn("w:color"), RED)
        border.attrib.pop(qn("w:themeColor"), None)
    header = section.header.paragraphs[0]
    header.add_run("CAN UNCLASSIFIED").font.size = Pt(8)
    header.add_run("\tOPERATION VANGUARD · COMPETITION PACKAGE").font.size = Pt(8)
    header.paragraph_format.tab_stops.add_tab_stop(Inches(2.45))
    footer = section.footer.paragraphs[0]
    footer.alignment = WD_ALIGN_PARAGRAPH.RIGHT
    footer.add_run("RMC ENGINEERING COMPETITION  |  ").font.size = Pt(8)
    field(footer, "PAGE")
    footer.add_run(" / ").font.size = Pt(8)
    field(footer, "NUMPAGES")
    first_footer = section.first_page_footer.paragraphs[0]
    first_footer.add_run("CAN UNCLASSIFIED").font.size = Pt(8)

    pages = (ROOT / "Competition-Package.md").read_text().split("<!-- page -->")
    for page_index, source in enumerate(pages):
        if page_index:
            document.add_page_break()
        lines = source.strip().splitlines()
        index = 0
        while index < len(lines):
            line = lines[index].strip()
            if not line:
                index += 1
                continue
            if line.startswith("|"):
                rows = []
                while index < len(lines) and lines[index].strip().startswith("|"):
                    cells = [cell.strip() for cell in lines[index].strip().strip("|").split("|")]
                    if not all(re.fullmatch(r":?-+:?", cell) for cell in cells):
                        rows.append(cells)
                    index += 1
                table(document, rows)
                continue
            if line.startswith("```"):
                index += 1
                while index < len(lines) and not lines[index].startswith("```"):
                    paragraph = document.add_paragraph()
                    run = paragraph.add_run(lines[index])
                    run.font.name = "Liberation Mono"
                    run.font.size = Pt(9)
                    index += 1
                index += 1
                continue
            heading = re.match(r"^(#{1,3}) (.*)$", line)
            if heading:
                level, title = len(heading[1]), heading[2]
                if page_index == 0:
                    style = {1: "Subtitle", 2: "Title", 3: "Subtitle"}[level]
                    paragraph = document.add_paragraph("Programming\nCompetition Package" if level == 2 else title, style)
                    if level == 1:
                        paragraph.paragraph_format.space_before = Pt(80)
                    if level == 2:
                        paragraph.paragraph_format.space_before = Pt(30)
                    if level == 3:
                        paragraph.paragraph_format.space_after = Pt(45)
                else:
                    document.add_heading(title, level)
                index += 1
                continue
            if line.startswith("- ") or re.match(r"^\d+\. ", line):
                ordered = not line.startswith("- ")
                paragraph = document.add_paragraph(style="List Number" if ordered else "List Bullet")
                paragraph.paragraph_format.space_after = Pt(4)
                inline(paragraph, re.sub(r"^(?:- |\d+\. )", "", line))
                index += 1
                continue
            parts = [line]
            while index + 1 < len(lines) and not lines[index].endswith("  ") and lines[index + 1].strip() and not re.match(r"^(?:#|- |\d+\. |\||```)", lines[index + 1].strip()):
                index += 1
                parts.append(lines[index].strip())
            if page_index == 0 and parts == ["CAN UNCLASSIFIED"]:
                index += 1
                continue
            paragraph = document.add_paragraph()
            inline(paragraph, " ".join(parts))
            index += 1
    document.core_properties.title = "Operation Vanguard — Programming Competition Package"
    document.core_properties.subject = "RMC Engineering Competition, 2026–2027; 50% simulator assessment"
    document.core_properties.keywords = "Operation Vanguard, competition, rubric, Lua, network defence"
    path = ROOT / "Operation-Vanguard-Competition-Package.docx"
    document.save(path)
    print(path)


if __name__ == "__main__":
    build()
