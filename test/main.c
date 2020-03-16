#include <stdlib.h>
#include <stdio.h>

#include "ermalloc.h"

#define START_FUNC \
    printf("===========================\n"); \
    printf("Start: %s\n", __FUNCTION__);

#define END_FUNC \
    printf("End: %s\n", __FUNCTION__); \
    printf("===========================\n");

void malloc_free_test(void)
{
    START_FUNC;
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
    END_FUNC;
}

void redundant_test(void)
{
    START_FUNC;

    struct er_policy_list p = {
        .policy = Redundancy,
        .policy_data = &(int){3},
        .next_policy = NULL
    };

    int* x = er_malloc(sizeof(int), &p);
    x[0] = 1;
    er_setup_policies(x);
    printf("x[0] = 0x%03x\n", x[0]);
    x[0] |= 1 << 4;
    printf("x[0] = 0x%03x\n", x[0]);
    int r = er_correct_buffer(x);
    printf("er_correct_buffer: %d, x[0] = 0x%03x\n", r, x[0]);
    printf("x[0] = 0x%03x\n", x[0]);
    x[0] |= 1 << 8;
    printf("x[0] = 0x%03x\n", x[0]);
    int x2 = 0;
    r = er_read_buf(x, &x2, 0, sizeof(int));
    printf("er_read_buf: %d, x2 = 0x%03x, x[0] = 0x%03x\n", r, x2, x[0]);
    er_free(x);

    END_FUNC;
}

void rs_test(void) {

    START_FUNC;

    struct er_policy_list p = {
        .policy = ReedSolomon,
        .policy_data = &(int){3},
        .next_policy = NULL
    };

    int* x = er_malloc(sizeof(int), &p);
    x[0] = 0b1010;
    er_setup_policies(x);
    printf("x[0] = %d\n", x[0]);
    x[0] = 0b1011;
    printf("x[0] = %d\n", x[0]);
    int r = er_correct_buffer(x);
    printf("x[0] = %d\n", x[0]);


    END_FUNC;
}

void rs_and_redundant_test(void) {

    START_FUNC;

    struct er_policy_list p = {
        .policy = ReedSolomon,
        .policy_data = &(int){3},
        .next_policy = NULL
    };

    struct er_policy_list p2 = {
        .policy = Redundancy,
        .policy_data = &(int){3},
        .next_policy = &p
    };

    int* x = er_malloc(sizeof(int), &p);
    x[0] = 0b1010;
    er_setup_policies(x);
    printf("x[0] = %d\n", x[0]);
    x[0] = 0b1011;
    printf("x[0] = %d\n", x[0]);
    int r = er_correct_buffer(x);
    printf("x[0] = %d\n", x[0]);

    END_FUNC;

}

int main(void)
{
    malloc_free_test();
    redundant_test();
    rs_test();
    rs_and_redundant_test();
    return 0;
}
