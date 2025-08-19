#include <stdio.h>

extern void __make_symbolic();

int main() {
    char i,j;
    __make_symbolic(sizeof(char), &i);
    __make_symbolic(sizeof(char), &j);
    if (i == 0) {
        if (j == '*') {}
        else {}
    } else {
        if (j == '$') { return 100; }
        else {}
    }
}

//_:_// expected exit status: 0
//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/symbolic_test_char.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_symbolic_test_char.c.ll)
//_:_// [+] Binary created (symbolic_test_char)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/symbolic_test_char

