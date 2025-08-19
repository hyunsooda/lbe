#include <stdio.h>

int main(int argc) {
    int a = 2;
    if (a == 1) {
        return a+1;
    } else if (a == 2) {
        return a+2;
    } else if (a == 3) {
        return a+3;
    } else {
        return a+4;
    }
}

//_:_// expected exit status: 4
//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/if.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_if.c.ll)
//_:_// [+] Binary created (if)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/if 
//_:_// +----------------------------+---------+-----------------+----------+---------------------------------+---------+-----------------+
//_:_// | File                       | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches              | % Lines | Uncovered lines |
//_:_// +----------------------------+---------+-----------------+----------+---------------------------------+---------+-----------------+
//_:_// | tests/inputs/coverage/if.c | 100.00  |                 | 33.33    | 6(7:F),9(8:T),10(12:F),12(10:T) | 63.64   | 6,9,10,12       |
//_:_// +----------------------------+---------+-----------------+----------+---------------------------------+---------+-----------------+

