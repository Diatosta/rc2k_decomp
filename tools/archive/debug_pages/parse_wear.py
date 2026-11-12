import idc, idautils, ida_bytes, json, collections
import os
OUT=os.path.join(os.environ.get("RC2K_DEBUG_PAGES", os.getcwd()), "wear_items.json")
ins=list(idautils.FuncItems(0x429B19))
items=[]; mode=None; scale=None; regs={}; pushes=[]; unknown=[]; pend={}
fmt={0x4203A3:"int",0x4203B4:"float",0x4203D6:"hex"}
for a in ins:
  m=idc.print_insn_mnem(a); o0=idc.print_operand(a,0); o1=idc.print_operand(a,1)
  v0=idc.get_operand_value(a,0); v1=idc.get_operand_value(a,1); t1=idc.get_operand_type(a,1)
  if m=="mov" and o0=="g_textMode": mode=v1
  elif m=="mov" and o0=="g_debugScale": scale=v1
  elif m in("pusha","popa","fld","fdivp","fistp","jns","js","cmp","retn","fild","and"): pass
  elif m=="push": pushes.append(v0 if idc.get_operand_type(a,0)==5 else ["mem",v0])
  elif m=="pop": pend.setdefault(v0,[]).append(pushes.pop())
  elif m=="fmul": pend["var"]=v1
  elif m=="mov" and o0 in("ecx","edx"): regs[o0]=v1
  elif m=="mov" and o0=="eax" and o1 in("dword_5C5FDC","dword_5C5FD8","dword_5C5FD4"): pass
  elif m=="mov" and o0=="eax" and t1==5: pass
  elif m=="mov" and o0=="eax" and t1==2: regs["eax"]=v1
  elif m=="mov" and o0 in("dword_5C5FDC","dword_5C5FE0"): pass
  elif m=="add" and o0=="eax": pend["h"]=v1
  elif m=="add" and o0=="dword_5C5FDC": pass
  elif m=="mov" and o0=="ebx" and t1==2: regs["ebx"]=v1
  elif m=="mov" and o0=="esi": regs["esi"]=["surf",regs["ebx"]]
  elif m=="lea" and o0=="esi": regs["esi"]=v1
  elif m=="call":
    if v0==0x41FF44:
      items.append(["bar",mode,pend[0x5C5FD4],pend[0x5C5FD8],pend[0x62403C],pend["var"],pend["h"]]); pend={}
    elif v0==0x420313:
      s=regs["esi"]
      if isinstance(s,list): items.append(["surf",mode,scale,regs["ecx"],regs["edx"],s[1]])
      else: items.append(["label",mode,scale,regs["ecx"],regs["edx"],idc.get_strlit_contents(s).decode()])
    elif v0 in fmt: items.append([fmt[v0],mode,scale,regs["ecx"],regs["edx"],regs["eax"]])
    else: items.append(["call",v0])
  else: unknown.append("%x %s"%(a,idc.GetDisasm(a)))
json.dump(items,open(OUT,"w"))
print(len(items), len(unknown), unknown[:20])
