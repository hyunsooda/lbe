#include <stdio.h>
#include <stdlib.h>

int main() {
    char *buf = (char *)malloc(8);
    for (int i=0; i<9; i++) { // OOB
        buf[i] = 'A';
    }
    free(buf);
    return 0;
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/oob-malloc2.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_oob-malloc2.c.ll)
//_:_// [+] Binary created (oob-malloc2)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/oob-malloc2 
//_:_// +---------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                            | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +---------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/asan/oob-malloc2.c | 100.00  |                 | 100.00   |                    | 100.00  |                 |
//_:_// +---------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+

//_:_// expected stderr:
//_:_// [ASAN] invalid memory access detected at tests/inputs/asan/oob-malloc2.c
//_:_//    5: main
//_:_//              at ./tests/inputs/asan/oob-malloc2.c:7:16
//_:_//    6: __libc_start_call_main
//_:_//    7: __libc_start_main_alias_2
//_:_//    8: _start

