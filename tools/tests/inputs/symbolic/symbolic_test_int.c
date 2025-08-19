#include <stdio.h>

extern void __make_symbolic();

int main() {
    int i,j;
    __make_symbolic(sizeof(int), &i);
    __make_symbolic(sizeof(int), &j);
    if (i == 0) {
        if (j == 123214125) { return 100; } 
        else {}
    } else {
        if (j == 1) {}
        else {}
    }
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/symbolic_test_int.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_symbolic_test_int.c.ll)
//_:_// [+] Binary created (symbolic_test_int)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/symbolic_test_int

