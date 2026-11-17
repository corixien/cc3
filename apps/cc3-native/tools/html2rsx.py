#!/usr/bin/env python3
"""Converts an XHTML snapshot (apps/web golden export, `<name>.body.xml`) into Dioxus RSX.
Usage: html2rsx.py <body.xml> [slot] > out.rs   (slot = data-slot value of the subtree to convert, default: <body>)
Attributes are emitted quoted, so any name works. The output is a starting point that is then
split into components with real data bindings; the golden comparison guards each step."""
import sys, re
import xml.etree.ElementTree as ET

SVG = "http://www.w3.org/2000/svg"
def local(tag): return tag.split('}')[-1]
def rs(s): return '"' + s.replace('\\', '\\\\').replace('"', '\\"').replace('\n', '\\n').replace('{', '{{').replace('}', '}}') + '"'

SKIP_ATTRS = {"xmlns", "xmlns:xlink"}
def attrs(el):
    out = []
    for k, v in el.attrib.items():
        k = local(k) if k.startswith('{') else k
        if k in SKIP_ATTRS or k.startswith('xmlns'): continue
        out.append(f'{rs(k).replace("{{","{").replace("}}","}")}: {rs(v)}')
    return out

IDENT = re.compile(r'^[a-zA-Z][a-zA-Z0-9_]*$')
def emit(el, depth, lines):
    pad = '    ' * depth
    tag = local(el.tag)
    name = tag if IDENT.match(tag) else None
    a = attrs(el)
    kids = list(el)
    text = (el.text or '')
    if name is None:
        name = tag.lower()
    head = f'{pad}{name} {{'
    if not a and not kids and not text.strip():
        lines.append(head + ' }'); return
    if len(a) <= 1 and not kids and text.strip():
        lines.append(head + ' ' + ', '.join(a + ([rs(text)] )) + ' }'); return
    lines.append(head)
    for x in a: lines.append(f'{pad}    {x},')
    if text.strip(): lines.append(f'{pad}    {rs(text)}')
    for kid in kids:
        emit(kid, depth + 1, lines)
        if (kid.tail or '').strip(): lines.append(f'{pad}    {rs(kid.tail)}')
    lines.append(f'{pad}}}')

def find(root, sel):
    """sel: a data-slot value, or `class:<prefix>`, optionally followed by `#<n>` (n-th match, 0-based)."""
    n = 0
    if '#' in sel: sel, n = sel.rsplit('#', 1); n = int(n)
    seen = 0
    for el in root.iter():
        hit = el.attrib.get('class', '').startswith(sel[6:]) if sel.startswith('class:') else el.attrib.get('data-slot') == sel
        if hit:
            if seen == n: return el
            seen += 1
    return None

tree = ET.parse(sys.argv[1]).getroot()
node = tree if len(sys.argv) <= 2 or sys.argv[2] in ('children', 'body') else find(tree, sys.argv[2])
lines = []
if len(sys.argv) > 2 and sys.argv[2] == 'children':
    for kid in node: emit(kid, 1, lines)
else:
    emit(node if len(sys.argv) <= 2 or sys.argv[2] != 'body' else node, 1, lines)
print('\n'.join(lines))
