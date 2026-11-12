import re,os,collections,sys
R=[(0x613828,0x614D38),(0x719FD8,0x71AFB8),(0x71AFB8,0x71C814)]
inr=lambda v:any(a<=v<b for a,b in R)
c=collections.Counter(); fl=collections.Counter(); ex=collections.defaultdict(list)
for root,_,fs in os.walk('src'):
  for f in fs:
    if not f.endswith('.rs') or f in('car_store.rs','tables_store.rs','work_store.rs','relocations.rs'): continue
    p=os.path.join(root,f); s=open(p).read()
    for m in re.finditer(r'0x([0-9A-Fa-f]{6})\b',s):
      v=int(m.group(1),16)
      if not inr(v): continue
      ls=s.rfind('\n',0,m.start())+1; le=s.find('\n',m.end()); line=s[ls:le].strip()
      if line.startswith('//'): k='comment'
      elif re.search(r'0x'+m.group(1)+r'\s*\+',line): k='offset+'
      elif 'rva(0x'+m.group(1) in line or 'rva(0x'+m.group(1).upper() in line: k='rva'
      elif re.match(r'(pub )?const',line): k='const'
      else:
        d=0;j=m.start()-1;name=None
        while j>0 and m.start()-j<300:
          ch=s[j]
          if ch in ')]': d+=1
          elif ch in '([':
            if d==0:
              if ch=='(':
                kk=j
                while kk>0 and (s[kk-1].isalnum() or s[kk-1]=='_'): kk-=1
                name=s[kk:j] or 'tuple'
              else: name='list'
              break
            d-=1
          elif ch in ';{}' and d==0: name='stmt'; break
          j-=1
        k='call:'+str(name)
      c[k]+=1; fl[p]+=1; ex[k].append(p[4:]+': '+line[:110])
print(sum(c.values()))
for k,v in c.most_common(25): print(v,k)
print()
for k,v in fl.most_common(25): print(v,k)
if len(sys.argv)>1:
  for l in ex[sys.argv[1]][:int(sys.argv[2])]: print(l)
