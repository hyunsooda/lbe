#include <stdio.h>
#include <pthread.h>
#include <unistd.h>

int counter = 0;
int n = 100;

pthread_mutex_t mutex1;

void* increment_counter(void* arg) {
    pthread_mutex_lock(&mutex1);
    for (int i = 0; i < n; i++) {
        counter++;
    }
    pthread_mutex_unlock(&mutex1);
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
    counter += 1;
    return 0;
}

//_:_// expected stdout:
//_:_// [+] compiled to IR (covout/missing_lock.c.ll)
//_:_// [+] IR file instrumented (covout/instrumented_missing_lock.c.ll)
//_:_// [+] Binary created (missing_lock)
//_:_// [+] You can run LD_LIBRARY_PATH=../bin/debug ./covout/missing_lock 
//_:_// [--------------------- Data race detected #0 ---------------------]
//_:_// variable name      = counter
//_:_// variable decl      = 5
//_:_// variable used line = 38
//_:_// [related locks]
//_:_//     - lock variable name = mutex1
//_:_//     - lock variable decl = 8
//_:_// 
//_:_// +----------------------------------+---------+-----------------+----------+-------------------------------------+---------+-------------------+
//_:_// | File                             | % Funcs | Uncovered Funcs | % Branch | Uncovered Branches                  | % Lines | Uncovered lines   |
//_:_// +----------------------------------+---------+-----------------+----------+-------------------------------------+---------+-------------------+
//_:_// | tests/inputs/race/missing_lock.c | 100.00  |                 | 60.00    | 23(26:F),27(30:F),31(34:F),35(38:F) | 68.00   | ...23,24,27,28,31 |
//_:_// +----------------------------------+---------+-----------------+----------+-------------------------------------+---------+-------------------+

