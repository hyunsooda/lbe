#include <stdio.h>

int main(int argc) {
    int a = 1;
    int b = 2;
    return a+b;
}

//_:_// expected exit status: 3
//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/simple.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_simple.c.ll)
//_:_// [+] Binary created (simple)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/simple 
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                           | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/coverage/simple.c | 100.00  |                 | NaN      |                    | 100.00  |                 |
//_:_// +--------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+

