import json, idc, idautils
import os
D=os.path.join(os.environ.get("RC2K_DEBUG_PAGES", os.getcwd()), "")
names={}
for a in idautils.FuncItems(0x429B19):
  m=idc.print_insn_mnem(a)
  if m in("mov","fmul") and idc.get_operand_type(a,1)==2:
    v=idc.get_operand_value(a,1)
    d=idc.GetDisasm(a)
    op=idc.print_operand(a,1)
    cmt=d.split(";",1)[1].strip() if ";" in d else ""
    names[hex(v)]=[op,cmt[:90]]
json.dump(names,open(D+"names.json","w"))
print(len(names))
