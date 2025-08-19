#include <stdio.h>
#include <stdlib.h>

void myfunc() {
    int* arr1 = (int*)malloc(sizeof(int) * 10);
    char* arr2 = (char*)malloc(sizeof(char) * 20);
    int a = 25;
    arr1[2] = 123;
    arr2[a] = 'a'; // OOB access
}

int main() {
    myfunc();
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/oob-malloc.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_oob-malloc.c.ll)
//_:_// [+] Binary created (oob-malloc)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/oob-malloc 
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                           | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/asan/oob-malloc.c | 100.00  |                 | NaN      |                    | 100.00  |                 |
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+

//_:_// expected stderr:
//_:_// [ASAN] invalid memory access detected at tests/inputs/asan/oob-malloc.c
//_:_//    5: myfunc
//_:_//              at ./tests/inputs/asan/oob-malloc.c:9:13
//_:_//    6: main
//_:_//              at ./tests/inputs/asan/oob-malloc.c:13:5
//_:_//    7: __libc_start_call_main
//_:_//    8: __libc_start_main_alias_2
//_:_//    9: _start

