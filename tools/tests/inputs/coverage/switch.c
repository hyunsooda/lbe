#include <stdio.h>

int main(int argc) {
    int a = 2;
    switch (a) {
        case 1:
            return a+1;
        case 2:
            return a+2;
        case 3:
            return a+3;
        default:
            return a+4;
    }
    // equiv with the one below
    /*
    if (a == 1) {
        return a+1;
    } else if (a == 2) {
        return a+1;
    } else if (a == 3) {
        return a+3;
    } else {
        return a+4;
    }
    */
}

//_:_// expected exit status: 4
//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/switch.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_switch.c.ll)
//_:_// [+] Binary created (switch)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/switch 
//_:_// +--------------------------------+---------+-----------------+----------+--------------------------+---------+-----------------+
//_:_// | File                           | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches       | % Lines | Uncovered lines |
//_:_// +--------------------------------+---------+-----------------+----------+--------------------------+---------+-----------------+
//_:_// | tests/inputs/coverage/switch.c | 100.00  |                 | 25.00    | 7(9:F),11(13:F),13(11:T) | 66.67   | 7,11,13         |
//_:_// +--------------------------------+---------+-----------------+----------+--------------------------+---------+-----------------+

