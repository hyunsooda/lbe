#include <stdio.h>
#include <stdlib.h>

void myfunc() {
    int arr[10];
    arr[7] = 123;
    int v = arr[15];
}

int main() {
    myfunc();
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/oob-stack.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_oob-stack.c.ll)
//_:_// [+] Binary created (oob-stack)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/oob-stack 
//_:_// +-------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                          | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +-------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/asan/oob-stack.c | 100.00  |                 | NaN      |                    | 100.00  |                 |
//_:_// +-------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+

//_:_// expected stderr:
//_:_// [ASAN] invalid memory access detected at tests/inputs/asan/oob-stack.c
//_:_//    5: myfunc
//_:_//              at ./tests/inputs/asan/oob-stack.c:7:13
//_:_//    6: main
//_:_//              at ./tests/inputs/asan/oob-stack.c:11:5
//_:_//    7: __libc_start_call_main
//_:_//    8: __libc_start_main_alias_2
//_:_//    9: _start

