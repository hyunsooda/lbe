#include <stdio.h>

extern void __make_symbolic();

int main() {
    int i,j;
    __make_symbolic(sizeof(int), &i);
    __make_symbolic(sizeof(int), &j);
    printf("%d, %d\n", i,j);
    if (i == 0) {
        if (j == 123214125) {
            printf("S1\n");
        } else {
            printf("S2\n");
        }
    } else {
        if (j != 88148128) {
            printf("S3\n");
        } else {
            printf("S4\n");
            return 123;
        }
    }
}
