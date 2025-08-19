#include <stdio.h>
#include <stdlib.h>

void myfunc() {
    int arr[10];
    arr[2] = 123;
    printf("%d\n", arr[2]);
}

int main() {
    myfunc();
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/safe-stack.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_safe-stack.c.ll)
//_:_// [+] Binary created (safe-stack)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/safe-stack 
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                           | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/asan/safe-stack.c | 100.00  |                 | NaN      |                    | 100.00  |                 |
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// 123

