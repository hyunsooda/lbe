#include <stdio.h>

#define FUZZ_LEN 10
#define ABNORMAL_EXIT 100

int main(int argc, char* argv[]) {
    char input[FUZZ_LEN];
    fgets(input, sizeof(input), stdin);
    printf("target run\n");

    if (input[0] == 'a') {
        if (input[1] == 'c' || input[1] == 'd' || input[1] == 'e' || input[1] == 'f') { // buggy path
            /* input[FUZZ_LEN] = 'a'; */
            return ABNORMAL_EXIT;
        }
    }
    return 0;
}
