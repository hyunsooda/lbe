#include <stdio.h>

#define ARR_SIZE 5

int get_odd_even_diff(int* arr, int size) {
    int even_cnt = 0;
    int odd_cnt = 0;
    for (int i=0; i<size; i++) {
        if (arr[i] % 2 == 0) {
            even_cnt++;
        } else {
            odd_cnt++;
        }
    }
    return odd_cnt - even_cnt;
}

int main() {
    /* int arr[ARR_SIZE] = {1,2,3,4,5}; */
    /* printf("%d\n", get_odd_even_diff(arr, ARR_SIZE)); */
    /* int arr[ARR_SIZE] = {1,3,5,7,9}; */
    /* printf("%d\n", get_odd_even_diff(arr, ARR_SIZE)); */
    int arr[ARR_SIZE] = {2,4,6,8,10};
    printf("%d\n", get_odd_even_diff(arr, ARR_SIZE));
}
