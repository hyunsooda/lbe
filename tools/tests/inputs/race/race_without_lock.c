#include <stdio.h>
#include <pthread.h>
#include <unistd.h>

int counter = 0;
int n = 100;

void* increment_counter(void* arg) {
    for (int i = 0; i < n; i++) {
        counter++;
    }
    return NULL;
}

int main() {
    pthread_t thread1, thread2;

    if (pthread_create(&thread1, NULL, increment_counter, NULL) != 0) {
        perror("Error creating thread 1");
        return 1;
    }
    if (pthread_create(&thread2, NULL, increment_counter, NULL) != 0) {
        perror("Error creating thread 2");
        return 1;
    }
    if (pthread_join(thread1, NULL) != 0) {
        perror("Error joining thread 1");
        return 1;
    }
    if (pthread_join(thread2, NULL) != 0) {
        perror("Error joining thread 2");
        return 1;
    }
    return 0;
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/race_without_lock.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_race_without_lock.c.ll)
//_:_// [+] Binary created (race_without_lock)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/race_without_lock 
//_:_// [--------------------- Data race detected #0 ---------------------]
//_:_// variable name      = counter
//_:_// variable decl      = 5
//_:_// variable used line = 10
//_:_// [related locks]
//_:_// 
//_:_// [--------------------- Data race detected #1 ---------------------]
//_:_// variable name      = counter
//_:_// variable decl      = 5
//_:_// variable used line = 10
//_:_// [related locks]
//_:_// 
//_:_// +---------------------------------------+---------+-----------------+----------+-------------------------------------+---------+-------------------+
//_:_// | File                                  | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches                  | % Lines | Uncovered lines   |
//_:_// +---------------------------------------+---------+-----------------+----------+-------------------------------------+---------+-------------------+
//_:_// | tests/inputs/race/race_without_lock.c | 100.00  |                 | 60.00    | 19(22:F),23(26:F),27(30:F),31(34:F) | 63.64   | ...19,20,23,24,27 |
//_:_// +---------------------------------------+---------+-----------------+----------+-------------------------------------+---------+-------------------+

