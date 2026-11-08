// A probe of the handheld's GPU (Mali-G31, libmali's fbdev EGL): can it draw to /dev/fb0, and how
// fast, on a load shaped like the game's frame (renderer plan, step 1). Not part of the game.
// Build: tools/anbernic/run.sh aarch64-linux-gnu-gcc -O2 -o target/gpu_probe
//        tools/anbernic/gpu_probe/gpu_probe.c -ldl
// Run on the device with the launcher stopped.
#include <dlfcn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

typedef void *P;
typedef int I;
typedef unsigned U;
#define EGL(ret, name, ...) static ret (*name)(__VA_ARGS__);
#define FUNCS \
  EGL(P, eglGetDisplay, P) EGL(I, eglInitialize, P, I *, I *) EGL(I, eglChooseConfig, P, const I *, P *, I, I *) \
  EGL(P, eglCreateWindowSurface, P, P, P, const I *) EGL(P, eglCreateContext, P, P, P, const I *) \
  EGL(I, eglMakeCurrent, P, P, P, P) EGL(I, eglSwapBuffers, P, P) EGL(I, eglSwapInterval, P, I) EGL(I, eglGetError, void) \
  EGL(I, eglBindAPI, U) EGL(const char *, eglQueryString, P, I) EGL(I, eglGetConfigAttrib, P, P, I, I *) \
  EGL(const unsigned char *, glGetString, U) EGL(U, glCreateShader, U) EGL(void, glShaderSource, U, I, const char **, const I *) \
  EGL(void, glCompileShader, U) EGL(void, glGetShaderiv, U, U, I *) EGL(void, glGetShaderInfoLog, U, I, I *, char *) \
  EGL(U, glCreateProgram, void) EGL(void, glAttachShader, U, U) EGL(void, glLinkProgram, U) EGL(void, glUseProgram, U) \
  EGL(void, glGenTextures, I, U *) EGL(void, glBindTexture, U, U) EGL(void, glTexImage2D, U, I, I, I, I, I, U, U, const void *) \
  EGL(void, glTexSubImage2D, U, I, I, I, I, I, U, U, const void *) EGL(void, glTexParameteri, U, U, I) \
  EGL(void, glGenerateMipmap, U) EGL(void, glGenBuffers, I, U *) EGL(void, glBindBuffer, U, U) \
  EGL(void, glBufferData, U, intptr_t, const void *, U) EGL(void, glEnableVertexAttribArray, U) \
  EGL(void, glVertexAttribPointer, U, I, U, unsigned char, I, const void *) EGL(void, glBindAttribLocation, U, U, const char *) \
  EGL(void, glViewport, I, I, I, I) EGL(void, glClearColor, float, float, float, float) EGL(void, glClear, U) \
  EGL(void, glEnable, U) EGL(void, glDisable, U) EGL(void, glBlendFunc, U, U) EGL(void, glDepthFunc, U) \
  EGL(void, glDrawArrays, U, I, I) EGL(void, glFinish, void) EGL(void, glReadPixels, I, I, I, I, U, U, void *) \
  EGL(I, glGetError, void) EGL(void, glGetIntegerv, U, I *) EGL(I, glGetUniformLocation, U, const char *) \
  EGL(void, glUniform1i, I, I)
FUNCS
#undef EGL

static double now(void) {
  struct timespec t;
  clock_gettime(CLOCK_MONOTONIC, &t);
  return t.tv_sec + t.tv_nsec / 1e9;
}

// The game's vertices come already transformed (x, y, z, 1/w), with two colours (fog in the second's alpha).
static const char *VS =
    "#version 300 es\nlayout(location=0) in vec4 p; layout(location=1) in vec2 t; layout(location=2) in vec4 c; out vec2 uv; out vec4 col; out float fog;\n"
    "void main(){ float w=1.0/p.w; gl_Position=vec4((p.x/320.0-1.0)*w,(1.0-p.y/240.0)*w,p.z*w,w); uv=t; col=c; fog=c.a; }\n";
static const char *FS =
    "#version 300 es\nprecision mediump float; uniform sampler2D s; in vec2 uv; in vec4 col; in float fog; out vec4 o;\n"
    "void main(){ vec4 x=texture(s,uv)*col; if(x.a<0.1) discard; o=vec4(mix(vec3(0.5,0.5,0.6),x.rgb,fog),x.a); }\n";
static const char *FS2D =
    "#version 300 es\nprecision mediump float; uniform sampler2D s; in vec2 uv; out vec4 o; void main(){ o=texture(s,uv).bgra; }\n";

static U shader(U kind, const char *text) {
  U s = glCreateShader(kind);
  glShaderSource(s, 1, &text, 0);
  glCompileShader(s);
  I ok = 0;
  glGetShaderiv(s, 0x8B81, &ok);
  if (!ok) {
    char log[1000];
    glGetShaderInfoLog(s, 1000, 0, log);
    printf("shader failed: %s\n", log);
    exit(1);
  }
  return s;
}
static U program(const char *fs) {
  U p = glCreateProgram();
  glAttachShader(p, shader(0x8B31, VS));
  glAttachShader(p, shader(0x8B30, fs));
  glLinkProgram(p);
  return p;
}

typedef struct { float x, y, z, w, u, v; unsigned char c[4]; } Vertex;
#define TEXTURES 16
#define TEX 256
#define TRIS_MAX 8000
static Vertex vertices[TRIS_MAX * 3];
static uint32_t pixels[TEX * TEX], screen[640 * 480];
static U textures[TEXTURES];

// `count` triangles of `size` pixels a side, spread over the screen, a texture change every `run`.
static void fill(int count, float size) {
  srand(1);
  for (int n = 0; n < count; n++) {
    float x = rand() % 640, y = rand() % 480, z = (rand() % 1000) / 1000.0f;
    float d[3][2] = {{0, 0}, {size, 0}, {0, size}};
    for (int k = 0; k < 3; k++) {
      Vertex *v = &vertices[n * 3 + k];
      v->x = x + d[k][0]; v->y = y + d[k][1]; v->z = z; v->w = 0.5f + z;
      v->u = d[k][0] / size; v->v = d[k][1] / size;
      memset(v->c, 200, 4);
    }
  }
  glBufferData(0x8892, count * 3 * sizeof(Vertex), vertices, 0x88E8);
}

static void scene(int count, int run) {
  glClear(0x4100);
  for (int n = 0, t = 0; n < count; n += run, t++) {
    glBindTexture(0x0DE1, textures[t % TEXTURES]);
    glDrawArrays(4, n * 3, (n + run > count ? count - n : run) * 3);
  }
}

int main(void) {
  void *mali = dlopen("/usr/lib/libmali.so", RTLD_NOW);
  if (!mali) { printf("no libmali: %s\n", dlerror()); return 1; }
#define EGL(ret, name, ...) *(void **)&name = dlsym(mali, #name); if (!name) { printf("missing %s\n", #name); return 1; }
  FUNCS
#undef EGL
  P display = eglGetDisplay(0);
  I major = 0, minor = 0;
  printf("eglInitialize %d", eglInitialize(display, &major, &minor));
  printf(" EGL %d.%d err %#x\n%s\n", major, minor, eglGetError(), eglQueryString(display, 0x3055 /*EXTENSIONS*/));
  const I want[] = {0x3024, 8, 0x3023, 8, 0x3022, 8, 0x3025, 24, 0x3033, 4 /*window*/, 0x3040, 0x40 /*ES3*/, 0x3038};
  P config = 0;
  I found = 0;
  eglChooseConfig(display, want, &config, 1, &found);
  printf("configs %d err %#x\n", found, eglGetError());
  if (!found) return 1;
  eglBindAPI(0x30A0);
  struct { unsigned short width, height; } window = {640, 480};
  P surface = eglCreateWindowSurface(display, config, &window, 0);
  printf("surface %p err %#x\n", surface, eglGetError());
  const I attributes[] = {0x3098, 3, 0x3038};
  P context = eglCreateContext(display, config, 0, attributes);
  printf("context %p err %#x\n", context, eglGetError());
  if (!surface || !context || !eglMakeCurrent(display, surface, surface, context)) { printf("make current failed %#x\n", eglGetError()); return 1; }
  printf("%s | %s | %s\n", glGetString(0x1F01), glGetString(0x1F02), glGetString(0x8B8C));
  I most = 0;
  glGetIntegerv(0x0D33, &most);
  printf("max texture %d\n", most);
  eglSwapInterval(display, 0);
  glViewport(0, 0, 640, 480);

  U p3d = program(FS), p2d = program(FS2D), vb;
  glGenBuffers(1, &vb);
  glBindBuffer(0x8892, vb);
  for (int n = 0; n < 3; n++) glEnableVertexAttribArray(n);
  glVertexAttribPointer(0, 4, 0x1406, 0, sizeof(Vertex), 0);
  glVertexAttribPointer(1, 2, 0x1406, 0, sizeof(Vertex), (void *)16);
  glVertexAttribPointer(2, 4, 0x1401, 1, sizeof(Vertex), (void *)24);
  for (int n = 0; n < TEX * TEX; n++) pixels[n] = 0xFF000000 | ((n % TEX) * 0x010101 ^ (n / TEX * 0x000100));
  glGenTextures(TEXTURES, textures);
  for (int t = 0; t < TEXTURES; t++) {
    glBindTexture(0x0DE1, textures[t]);
    glTexImage2D(0x0DE1, 0, 0x1908, TEX, TEX, 0, 0x1908, 0x1401, pixels);
    glGenerateMipmap(0x0DE1);
    glTexParameteri(0x0DE1, 0x2801, 0x2701);
    glTexParameteri(0x0DE1, 0x2800, 0x2601);
  }
  glEnable(0x0B71);
  glDepthFunc(0x0203);
  glClearColor(0.2f, 0.3f, 0.5f, 1);
  glUseProgram(p3d);

  // Loads: triangles, side in pixels, triangles a texture change.
  int loads[][3] = {{500, 30, 10}, {1500, 30, 10}, {3000, 30, 10}, {3000, 60, 10}, {6000, 30, 5}, {8000, 15, 5}};
  for (unsigned l = 0; l < sizeof loads / sizeof *loads; l++) {
    fill(loads[l][0], loads[l][1]);
    double begun = now(), work = 0;
    int frames = 0;
    while (now() - begun < 3) {
      double t = now();
      scene(loads[l][0], loads[l][2]);
      work += now() - t; // what the game's thread pays to submit
      eglSwapBuffers(display, surface);
      frames++;
    }
    printf("load %5d tris, side %3d, %3d a texture: %5.1f fps, submit %5.2f ms\n", loads[l][0], loads[l][1], loads[l][2], frames / (now() - begun), work * 1000 / frames);
  }
  // Blending on, as the transparent part of a frame would be.
  glEnable(0x0BE2);
  glBlendFunc(0x0302, 0x0303);
  fill(1500, 60);
  { double begun = now(); int frames = 0;
    while (now() - begun < 3) { scene(1500, 10); eglSwapBuffers(display, surface); frames++; }
    printf("blended 1500 tris side 60: %.1f fps\n", frames / (now() - begun)); }
  glDisable(0x0BE2);

  // A texture changed every frame (the mud).
  { double begun = now(), work = 0; int frames = 0;
    while (now() - begun < 2) {
      double t = now();
      glBindTexture(0x0DE1, textures[0]);
      glTexSubImage2D(0x0DE1, 0, 0, 0, TEX, TEX, 0x1908, 0x1401, pixels);
      work += now() - t;
      scene(500, 10); eglSwapBuffers(display, surface); frames++; }
    printf("256x256 upload each frame: %.2f ms, %.1f fps\n", work * 1000 / frames, frames / (now() - begun)); }

  // The CPU's 2D: a whole 640x480 picture up as a texture and drawn, each frame (menus, the panel's blits).
  glUseProgram(p2d);
  U full;
  glGenTextures(1, &full);
  glBindTexture(0x0DE1, full);
  glTexImage2D(0x0DE1, 0, 0x1908, 640, 480, 0, 0x1908, 0x1401, 0);
  glTexParameteri(0x0DE1, 0x2801, 0x2601);
  glTexParameteri(0x0DE1, 0x2800, 0x2601);
  Vertex quad[6];
  float corners[6][2] = {{0, 0}, {1, 0}, {0, 1}, {1, 0}, {1, 1}, {0, 1}};
  for (int k = 0; k < 6; k++) { quad[k] = (Vertex){corners[k][0] * 640, corners[k][1] * 480, 0, 1, corners[k][0], corners[k][1], {255, 255, 255, 255}}; }
  glBufferData(0x8892, sizeof quad, quad, 0x88E4);
  glDisable(0x0B71);
  { double begun = now(), work = 0; int frames = 0;
    while (now() - begun < 3) {
      for (int y = 0; y < 480; y++) for (int x = 0; x < 640; x++) screen[y * 640 + x] = (x + frames) * 0x010203 ^ (y << 8);
      double t = now();
      glTexSubImage2D(0x0DE1, 0, 0, 0, 640, 480, 0x1908, 0x1401, screen);
      work += now() - t;
      glDrawArrays(4, 0, 6);
      eglSwapBuffers(display, surface);
      frames++; }
    printf("640x480 picture up and shown: upload %.2f ms, %.1f fps\n", work * 1000 / frames, frames / (now() - begun)); }

  // The screen back to the CPU (`copy_screen_to_system_texture`).
  { double begun = now(); int n = 0;
    while (n < 30) { glDrawArrays(4, 0, 6); glReadPixels(0, 0, 640, 480, 0x1908, 0x1401, screen); n++; }
    printf("640x480 read back: %.2f ms\n", (now() - begun) * 1000 / n);
    glEnable(0x0B71); }

  // Something to see: a few seconds of the scene.
  glUseProgram(p3d);
  fill(1500, 40);
  { double begun = now(); while (now() - begun < 3) { scene(1500, 10); eglSwapBuffers(display, surface); } }
  printf("gl error %#x, egl error %#x\n", glGetError(), eglGetError());
  return 0;
}
