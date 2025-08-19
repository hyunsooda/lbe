#include <stdio.h>
#include <stdlib.h>

void myfunc() {
    int* arr = (int*)malloc(sizeof(int) * 10);
    arr[2] = 123;
    printf("%d\n", arr[2]);
}

int main() {
    myfunc();
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/safe-malloc.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_safe-malloc.c.ll)
//_:_// [+] Binary created (safe-malloc)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/safe-malloc 
//_:_// +---------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                            | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +---------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/asan/safe-malloc.c | 100.00  |                 | NaN      |                    | 100.00  |                 |
//_:_// +---------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// 123

