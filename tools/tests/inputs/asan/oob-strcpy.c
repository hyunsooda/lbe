#include <string.h>

void myfunc() {
    char buffer[10];
    strcpy(buffer, "aaaaaaaaaa");
}

int main() {
   myfunc();
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/oob-strcpy.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_oob-strcpy.c.ll)
//_:_// [+] Binary created (oob-strcpy)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/oob-strcpy 
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                           | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/asan/oob-strcpy.c | 100.00  |                 | NaN      |                    | 100.00  |                 |
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+

//_:_// expected stderr:
//_:_// [ASAN] invalid memory access detected at libc::strcpy
//_:_//    5: strcpy
//_:_//    6: myfunc
//_:_//              at ./tests/inputs/asan/oob-strcpy.c:5:5
//_:_//    7: main
//_:_//              at ./tests/inputs/asan/oob-strcpy.c:9:4
//_:_//    8: __libc_start_call_main
//_:_//    9: __libc_start_main_alias_2
//_:_//   10: _start

