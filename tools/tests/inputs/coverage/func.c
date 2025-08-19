#include <stdio.h>

int add(int v1, int v2) {
    return v1+v2;
}

int sub(int v1, int v2) {
    return v1-v2;
}

int mul_not_called(int v1, int v2) {
    return v1*v2;
}

int main(int argc) {
    int a = 1;
    int b = 2;
    add(a, b);
    sub(a, b);
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/func.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_func.c.ll)
//_:_// [+] Binary created (func)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/func 
//_:_// +------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                         | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/coverage/func.c | 75.00   | 11              | NaN      |                    | 84.62   | 11,12           |
//_:_// +------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+

