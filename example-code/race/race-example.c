#include <stdio.h>
#include <pthread.h>
#include <unistd.h>

volatile int counter = 0;
int n = 20000;

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

    printf("Expected counter value: %d\n", 2 * n);
    printf("Actual counter value: %d\n", counter);

    return 0;
}
