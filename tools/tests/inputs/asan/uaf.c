#include <stdio.h>
#include <stdlib.h>

int main() {
    int* arr = (int*)malloc(sizeof(int) * 10);
    free(arr);
    arr[2] = 123; // UAF
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/uaf.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_uaf.c.ll)
//_:_// [+] Binary created (uaf)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/uaf 
//_:_// +-------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                    | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +-------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/asan/uaf.c | 100.00  |                 | NaN      |                    | 100.00  |                 |
//_:_// +-------------------------+---------+-----------------+----------+--------------------+---------+-----------------+

//_:_// expected stderr:
//_:_// [ASAN] invalid memory access detected at tests/inputs/asan/uaf.c
//_:_//    5: main
//_:_//              at ./tests/inputs/asan/uaf.c:7:12
//_:_//    6: __libc_start_call_main
//_:_//    7: __libc_start_main_alias_2
//_:_//    8: _start

