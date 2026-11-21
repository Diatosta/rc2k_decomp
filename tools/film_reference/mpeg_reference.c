// What FFmpeg's decoders give for an elementary stream, the way the film player uses them (the
// parser cuts the stream, each packet is decoded, an empty one ends it):
//   mpeg_reference mpeg1video|mp2 <stream> <out> <list>
// <out>: the pictures (yuv420p, the planes without padding) or the samples (16 bits, little
// endian, the two channels interleaved); <list>: for each packet its bytes and the frames it gave.
// FFmpeg's functions are declared here as the program's own wrapper declares them.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
typedef struct { uint8_t *data[8]; int linesize[8]; void *ext; int width,height,nb_samples,format; } Frame;
void *avcodec_find_decoder_by_name(const char*); void *avcodec_alloc_context3(void*); int avcodec_open2(void*,void*,void*);
void *av_packet_alloc(void); int av_packet_from_data(void*,uint8_t*,int); void av_packet_unref(void*);
int avcodec_send_packet(void*,void*); int avcodec_receive_frame(void*,void*); void *av_parser_init(int);
int av_parser_parse2(void*,void*,uint8_t**,int*,const uint8_t*,int,int64_t,int64_t,int64_t);
void *av_frame_alloc(void); void *av_malloc(size_t);
static FILE *out,*sizes; static int video; static long frames;
static void got(Frame *f){ frames++;
  if(video){ for(int p=0;p<3;p++){int w=p?f->width/2:f->width,h=p?f->height/2:f->height; for(int y=0;y<h;y++) fwrite(f->data[p]+y*f->linesize[p],1,w,out);} }
  else { for(int i=0;i<f->nb_samples;i++) for(int c=0;c<2;c++) fwrite(f->data[c]+2*i,2,1,out); if(frames==1) fprintf(stderr,"fmt %d n %d\n",f->format,f->nb_samples);} }
static void dec(void*ctx,void*pkt,void*fr,uint8_t*d,int n){
  int s; if(d){uint8_t*m=av_malloc(n+64);memcpy(m,d,n);memset(m+n,0,64);av_packet_from_data(pkt,m,n);s=avcodec_send_packet(ctx,pkt);av_packet_unref(pkt);} else s=avcodec_send_packet(ctx,NULL);
  if(s<0) fprintf(stderr,"send %d\n",s);
  while(avcodec_receive_frame(ctx,fr)>=0) got(fr); }
int main(int c,char**v){ video=!strcmp(v[1],"mpeg1video");
  FILE*in=fopen(v[2],"rb"); fseek(in,0,SEEK_END); long n=ftell(in); rewind(in); uint8_t*b=malloc(n+64); memset(b,0,n+64); fread(b,1,n,in);
  out=fopen(v[3],"wb"); sizes=fopen(v[4],"w");
  void*d=avcodec_find_decoder_by_name(v[1]); void*ctx=avcodec_alloc_context3(d); if(avcodec_open2(ctx,d,NULL)<0) return 1;
  void*pkt=av_packet_alloc(),*fr=av_frame_alloc(),*ps=av_parser_init(video?1:0x15000);
  long at=0; long np=0;
  for(;;){ long rest=n-at; uint8_t*data=NULL; int size=0;
    int used=av_parser_parse2(ps,ctx,&data,&size,rest?b+at:NULL,(int)rest,INT64_MIN,INT64_MIN,0);
    if(size>0&&data){ long before=frames; uint8_t*copy=malloc(size); memcpy(copy,data,size); dec(ctx,pkt,fr,copy,size); fprintf(sizes,"%d %ld\n",size,frames-before); free(copy); np++; }
    if(!rest&&size<=0) break; at+=used>0?used:0; if(used<=0&&rest&&size<=0) break; }
  dec(ctx,pkt,fr,NULL,0);
  fprintf(stderr,"%ld packets %ld frames\n",np,frames); return 0; }
