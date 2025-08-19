#include <stdio.h>

int getValue(int input) {
    switch (input) {
        case 1:
            return 10;
        case 2:
            return 20;
        case 3:
            return 30;
        case 4:
            return 40;
        default:
            return -1;
    }
}

int main() {
    return getValue(4);
}

//_:_// expected exit status: 40
//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/switch2.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_switch2.c.ll)
//_:_// [+] Binary created (switch2)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/switch2 
//_:_// +---------------------------------+---------+-----------------+----------+------------------------+---------+-----------------+
//_:_// | File                            | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches     | % Lines | Uncovered lines |
//_:_// +---------------------------------+---------+-----------------+----------+------------------------+---------+-----------------+
//_:_// | tests/inputs/coverage/switch2.c | 100.00  |                 | 25.00    | 6(8:F),8(6:T),10(12:F) | 63.64   | 6,8,10,14       |
//_:_// +---------------------------------+---------+-----------------+----------+------------------------+---------+-----------------+

