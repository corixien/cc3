#!/usr/bin/env python3
"""Extracts command palette row templates from the golden fragments into assets/palette/."""
import json, re
from lxml import html as H, etree

def rows(name):
    doc = H.fromstring(open(f'assets/fragments/{name}.overlay.html').read())
    out = []
    for el in doc.iter():
        slot = el.get('data-slot')
        if slot in ('command-item', 'command-group-label'):
            out.append((slot, el))
    return out

def ser(el):
    el.tail = None
    return etree.tostring(el, method='html', encoding='unicode')

def text(el):
    return re.sub(r'\s+', ' ', ''.join(el.itertext())).strip()

idx = {'actions': [], 'thread': None, 'group': None, 'settings': None}
for slot, el in rows('overlay-command-palette'):
    if slot == 'command-group-label':
        if idx['group'] is None: idx['group'] = ser(el)
        continue
    t = text(el)
    h = ser(el)
    if 'Thread thread-other' in t:
        idx['thread'] = h
    else:
        idx['actions'].append({'text': t, 'html': h})
for slot, el in rows('overlay-command-palette-query'):
    if slot == 'command-item' and 'Thread details' in text(el):
        idx['settings'] = ser(el)
json.dump(idx, open('assets/palette/rows.json', 'w'), ensure_ascii=False)
print(len(idx['actions']), [a['text'][:24] for a in idx['actions']], bool(idx['thread']), bool(idx['settings']))
