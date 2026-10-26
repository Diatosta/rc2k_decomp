# Takes out of `use` lists the names rustc cannot resolve (constants a converter removed):
# reads clippy's short output on stdin (`error[E0432]: unresolved import(s) `a::b::N``).
# A `use` left empty goes. Run from the repository root.
import re, sys, collections
gone = collections.defaultdict(set)
for l in sys.stdin:
    m = re.match(r'(src/[^:]+):\d+:\d+: error\[E0432\]: unresolved imports? (.*)', l)
    if not m: continue
    for path in re.findall(r'`([\w:]+)`', m.group(2).split(': no ')[0]):
        if '::' in path: gone[m.group(1)].add(tuple(path.rsplit('::', 1)))
for p, items in gone.items():
    s = open(p).read()
    for mod, name in items:
        mod = re.sub(r'^crate::', '', mod)
        # `use crate::mod::NAME;`
        s = re.sub(r'^use crate::' + re.escape(mod) + r'::' + name + r';\n', '', s, flags=re.M)
        # inside braces of `use crate::mod::{..}` or a nested `mod_tail::{..}`
        def fix(m):
            body = [x.strip() for x in m.group(2).split(',') if x.strip()]
            body = [x for x in body if x != name]
            if not body: return ''
            return m.group(1) + '{' + ', '.join(body) + '}' + m.group(3)
        tail = mod.split('::')[-1]
        s = re.sub(r'((?:use crate::' + re.escape(mod) + r'::|\b' + tail + r'::))\{([^{}]*)\}(;?)', fix, s)
        s = re.sub(r'\b' + tail + r'::' + name + r',\s*', '', s)
    s = re.sub(r'^use [\w:]+::;\n', '', s, flags=re.M)
    open(p, 'w').write(s); print(p, sorted(n for _, n in items))
