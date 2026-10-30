import json, idc, ida_bytes
D="/tmp/claude-1000/-mnt-discoII-RustRoverProjects-rc2k-hook/e12e57fe-1a50-453f-b240-d0b96a47f5c0/scratchpad/"
it=json.load(open(D+"wear_items.json"))
rows=[]; label=""
for x in it[:-1]:
  if x[0]=="label": label=x[5]; continue
  if x[0]=="bar": continue
  at=x[5]
  head=idc.get_item_head(at)
  name=idc.get_name(head) or ""
  off=at-head
  # struct member name via disasm of first ref is hard; use get_name with offset
  full=idc.get_name_expr(0, 0, at) if hasattr(idc,'get_name_expr') else ""
  cmt=(idc.get_cmt(head,0) or idc.get_cmt(head,1) or "")[:70]
  rows.append([label,x[0],hex(at),x[3],x[4],"%s+%#x"%(name,off) if off else name,cmt])
  label=""
json.dump(rows,open(D+"rows.json","w"))
print(len(rows))
