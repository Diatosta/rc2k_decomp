# An MPEG program stream's first video and audio streams as the film player takes them apart:
#   demux.py <film.MPG> <out>   writes <out>.m1v and <out>.mp2
import sys,struct
b=open(sys.argv[1],'rb').read()
video=bytearray();audio=bytearray();at=0;n=len(b)
while at+4<=n:
    code=struct.unpack('>I',b[at:at+4])[0]
    if code>>8!=1: at+=1; continue
    c=code&0xFF
    if c==0xB9: break
    if c==0xBA:
        mpeg1 = at+4<n and b[at+4]>>4==2
        at += 12 if mpeg1 else 14+((b[at+13] if at+13<n else 0)&7)
    elif c>=0xBB:
        ln=struct.unpack('>H',b[at+4:at+6])[0]
        end=min(at+6+ln,n)
        if c in (0xC0,0xE0):
            p=at+6
            while p<n and b[p]==0xFF: p+=1
            if p<n and b[p]&0xC0==0x40: p+=2
            x=b[p]&0xF0 if p<n else None
            p+= 5 if x==0x20 else 10 if x==0x30 else 1
            d=b[min(p,end):end]
            (video if c==0xE0 else audio).extend(d)
        at=end
    else: at+=4
open(sys.argv[2]+'.m1v','wb').write(video);open(sys.argv[2]+'.mp2','wb').write(audio)
print(len(video),len(audio))
