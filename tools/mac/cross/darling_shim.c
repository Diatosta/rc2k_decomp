// Linked into Darling test builds only (`RC2K_MAC_WEAK_FRAMEWORKS=1`, cc.sh): Darling's
// clock_get_time on the calendar clock never returns (darlingserver, 2026-11-08), and SDL
// reads the wall clock through it on Apple. The calendar clock from clock_gettime instead;
// every other clock to the real call.
#include <dlfcn.h>
#include <mach/clock.h>
#include <mach/mach.h>
#include <math.h>
#include <time.h>

kern_return_t clock_get_time(clock_serv_t clock, mach_timespec_t *now) {
    static clock_serv_t calendar;
    if (calendar == 0) {
        host_get_clock_service(mach_host_self(), CALENDAR_CLOCK, &calendar);
    }
    if (clock == calendar) {
        struct timespec t;
        if (clock_gettime(CLOCK_REALTIME, &t) != 0) {
            return KERN_FAILURE;
        }
        now->tv_sec = (unsigned int)t.tv_sec;
        now->tv_nsec = (clock_res_t)t.tv_nsec;
        return KERN_SUCCESS;
    }
    kern_return_t (*real)(clock_serv_t, mach_timespec_t *) =
        (kern_return_t (*)(clock_serv_t, mach_timespec_t *))dlsym(RTLD_NEXT, "clock_get_time");
    return real(clock, now);
}

// Darling's __exp10 is `pow(10, x)`, which clang folds back into __exp10: built as a jump to
// itself, it never returns (2026-11-08); its __exp10f goes through it. LLVM lowers Rust's
// `10f32.powf(x)` to __exp10f on Apple. pow / powf through a volatile pointer, which the
// compiler cannot fold.
extern double pow(double, double);
double __exp10(double x) {
    double (*volatile real_pow)(double, double) = pow;
    return real_pow(10.0, x);
}

extern float powf(float, float);
float __exp10f(float x) {
    float (*volatile real_powf)(float, float) = powf;
    return real_powf(10.0f, x);
}

// Darling's __sincosf_stret / __sincos_stret are sin and cos taken separately, which clang
// folds back into the same call: they call themselves until the stack runs out (2026-11-08).
// LLVM lowers a Rust `sin_cos` (and a sin and cos of one value) to them on Apple.
extern float sinf(float), cosf(float);
extern double sin(double), cos(double);
struct __float2 __sincosf_stret(float x) {
    float (*volatile real_sinf)(float) = sinf;
    float (*volatile real_cosf)(float) = cosf;
    struct __float2 both = {real_sinf(x), real_cosf(x)};
    return both;
}
struct __double2 __sincos_stret(double x) {
    double (*volatile real_sin)(double) = sin;
    double (*volatile real_cos)(double) = cos;
    struct __double2 both = {real_sin(x), real_cos(x)};
    return both;
}
