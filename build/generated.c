#include "../tol_helper.h"
#include <stdio.h>
#include <stdlib.h>
typedef struct Tao {
  int32_t edad;
  uint32_t tuig;
} Tao;
typedef struct Hayop {
  uint8_t edad;
} Hayop;
DEFINE_TOL_ARRAY_STRUCT(int32_t)
DEFINE_TOL_ARRAY_STRUCT(TOL_Array_int32_t)
void gen2__i32__Tao(int32_t a, Tao b) { b.edad; }
int32_t generic__i32__dobletang(int32_t a, double b) { return a; }
int32_t generic__i32__i32(int32_t a, int32_t b) { return a; }
Tao bago(int32_t edad, uint32_t tuig) {
  return (struct Tao){.edad = edad, .tuig = tuig};
}
uint32_t get_tuig(Tao ako) { ako.tuig; }
int32_t idagdag_sa(int32_t ako, int32_t iba) { return (ako + iba); }
int32_t wala_lang() { return 42; }
void __TOL_main__() {
  Tao const tao = bago(8, 10);
  int32_t const edad = tao.edad;
  uint32_t const tuig = (get_tuig(tao));
  int32_t numero = 1;
  numero = 2;
  int32_t const dagdag = numero;
  int32_t const resulta = (idagdag_sa(dagdag, 1));
  int32_t const res = (wala_lang() - 42);
  TOL_Array_TOL_Array_int32_t const array = (TOL_Array_TOL_Array_int32_t){
      .data =
          (TOL_Array_int32_t[]){
              (TOL_Array_int32_t){.data = (int32_t[]){1, 2, 3}, .len = 3},
          },
      .len = 1};
  TOL_Array_int32_t const another_arr =
      (TOL_Array_int32_t){.data = (int32_t[]){2, 0, 0, 0, 0}, .len = 5};
  const TOL_Array_TOL_Array_int32_t *const ptr = (&array);
  for (size_t i = 0; i <= 12; i++) {
    i;
  }
  (generic__i32__i32(5, 6));
  (generic__i32__dobletang(12, 12.2));
  (gen2__i32__Tao(5, tao));
  (generic__i32__i32(1, 2));
  exit(0);
}
int main() {
  __TOL_main__();
  return 0;
}