import re
import os
R = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'src') + '/'
s=open(R+'marks/store.rs').read()
m=re.search(r'pub const CARVED: &\[\(usize, usize\)\] = &\[\n(.*?)\];',s,re.S)
items=[]
for line in m.group(1).strip().split('\n'):
    mm=re.match(r'\s*\((0x[0-9A-Fa-f]+), (0x[0-9A-Fa-f]+)\),\s*//\s*`([^`]+)`',line)
    items.append((int(mm.group(1),16),int(mm.group(2),16),mm.group(3)))
items.sort()
S=0x764F40;E=0x93CCC8
# rewrite CARVED sorted
body=''.join('    (0x%X, 0x%X), // `%s`\n'%it for it in items)
s=s.replace(m.group(0),'pub const CARVED: &[(usize, usize)] = &[\n'+body+'];')
segs=[];pos=S
for a,b,_ in items:
    assert a>=pos,(hex(a),hex(pos))
    if a>pos: segs.append((pos,a))
    pos=b
if pos<E: segs.append((pos,E))
def lit(x): return 'START' if x==S else 'END' if x==E else '0x%X'%x
sb=''.join('    (%s, %s),\n'%(lit(a),lit(b)) for a,b in segs)
s=re.sub(r'pub const SEGS: &\[\(usize, usize\)\] = &\[.*?\];','pub const SEGS: &[(usize, usize)] = &[\n'+sb+'];',s,flags=re.S)
open(R+'marks/store.rs','w').write(s)
mv=open(R+'moved.rs').read()
ma=mv.index('    // marks: 0x764F40 .. 0x93CCC8 (`marks::store`)')
mb=mv.index('];',ma)
out='    // marks: 0x764F40 .. 0x93CCC8 (`marks::store`), less the ranges that left it (`CARVED`),\n    // in the segments of `SEGS`; then the ranges that left it, each its own typed static\n'
for i in range(len(segs)):
    out+='''    Moved {
        ida: crate::marks::store::SEGS[%d].0,
        len: crate::marks::store::SEGS[%d].1 - crate::marks::store::SEGS[%d].0,
        home: crate::marks::store::home_of::<{ crate::marks::store::SEGS[%d].0 }>,
    },
'''%(i,i,i,i)
for a,bb,ty in items:
    path='crate::'+ty if not ty.startswith('crate::') else ty
    out+='''    Moved {
        ida: %s::START,
        len: %s::LEN,
        home: %s::home,
    },
'''%(path,path,path)
mv=mv[:ma]+out+mv[mb:]
open(R+'moved.rs','w').write(mv)
print(len(segs),'segments',len(items),'pieces')
