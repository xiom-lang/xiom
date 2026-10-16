/* m348 fixture host: hands out raw C function addresses through TYPED
 * getters (the C-typedef loader shape: the declared return IS a fn pointer),
 * plus direct-extern and integer-address controls for the same functions.
 * The XIOM side binds the getter's return WITHOUT an `as` cast. */
#include <math.h>
#include <stdint.h>

static double mt348_add_impl(double a, double b) { return a + b; }

double mt348_cos_direct(double x) { return cos(x); }

long long mt348_cos_addr(void) { return (long long)(double (*)(double))cos; }

typedef double (*mt348_d1_f)(double);
typedef double (*mt348_d2_f)(double, double);
mt348_d1_f mt348_get_cos_fn(void) { return cos; }
mt348_d2_f mt348_get_add_fn(void) { return mt348_add_impl; }

static const char* mt348_label(void) { return "label-ok"; }
const char* (*mt348_get_label_fn(void))(void) { return mt348_label; }
