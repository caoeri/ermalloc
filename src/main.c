#include <stdlib.h>
#include <stdio.h>

#include "ermalloc.h"

void redundant_test(void)
{
    struct er_policy_list p = {
        .policy = Redundancy,
        .policy_data = &(int){3},
        .next_policy = NULL
    };

    int* x = er_malloc(sizeof(int), &p);
    x[0] = 1;
    x[1] = 2;
    x[2] = 2;
    x[3] = 2;
    er_enforce_policies(x);
    printf("%d %d %d %d\n", x[0], x[1], x[2], x[3]);
    er_free(x);
}

int main(void)
{
    int* x = er_malloc(123, NULL);
    printf("er_malloc(123, NULL)\n");
    x[12] = 7;
    x = er_realloc(x, 4096, NULL);
    printf("er_realloc(4096, NULL)\n");
    printf("x[12] = %d\n", x[12]);
    x[234] = 9;
    x = er_realloc(x, 2048, NULL);
    printf("er_realloc(2048, NULL)\n");
    printf("x[12] = %d\n", x[12]);
    printf("x[234] = %d\n", x[234]);
    er_free(x);
    redundant_test();
    return 0;
}
