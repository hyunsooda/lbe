#include <stdio.h>

int add() {
    int v1 = 1;
    int v2 = 1;
    int v3 = 1;
    int v4 = 1;
    int v5 = 1;
    int v6 = 1;
    int v7 = 1;
    return v1+v2+v3+v4+v5+v6+v7;
}

int main(int argc) {
    return 0;
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/many_uncvoered.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_many_uncvoered.c.ll)
//_:_// [+] Binary created (many_uncvoered)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/many_uncvoered 
//_:_// +----------------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | File                                   | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches | % Lines | Uncovered lines |
//_:_// +----------------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+
//_:_// | tests/inputs/coverage/many_uncvoered.c | 50.00   | 3               | NaN      |                    | 25.00   | ...3,4,5,6,7    |
//_:_// +----------------------------------------+---------+-----------------+----------+--------------------+---------+-----------------+

