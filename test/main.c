#include <stdlib.h>
#include <stdio.h>
#include <string.h>

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
    int x2 = 0;
    x[0] = 0b1011;
    r = er_read_buf(x, &x2, 0, sizeof(int));
    printf("x[0] = %d\n", x[0]);
    int x3 = 5;
    er_write_buf(x, &x3, 0, sizeof(int));
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

void encryption_test(void) {

    START_FUNC;

    struct er_policy_list p = {
        .policy = Encrypted,
        .policy_data = NULL,
        .next_policy = NULL
    };

    int* x = er_malloc(7*sizeof(int), &p);
    for (int i = 0; i < 7; i++) {
        x[i] = i+1;
    }
    for (int i = 0; i < 7; i++) {
        printf("x[%d] = %d\n", i, x[i]);
    }
    er_setup_policies(x);
    for (int i = 0; i < 7; i++) {
        printf("x[%d] = %d\n", i, x[i]);
    }

    int x2[7];
    er_read_buf(x, &x2, 0, 7*sizeof(int));
    for (int i = 0; i < 7; i++) {
        printf("x[%d] = %d\n", i, x2[i]);
    }

    int x3[7];
    for (int i = 0; i < 7; i++) {
        x3[i] = i+10;
    }
    er_write_buf(x, &x3, 0, 7*sizeof(int));
    for (int i = 0; i < 7; i++) {
        printf("x[%d] = %d\n", i, x[i]);
    }

    END_FUNC;

}

void combined_test(void) {

    START_FUNC;

    struct er_policy_list p = {
        .policy = Encrypted,
        .policy_data = NULL,
        .next_policy = NULL
    };

    struct er_policy_list p2 = {
        .policy = ReedSolomon,
        .policy_data = &(int){3},
        .next_policy = &p
    };

    struct er_policy_list p3 = {
        .policy = Redundancy,
        .policy_data = &(int){3},
        .next_policy = &p2
    };

    size_t len = 7 * sizeof(int);

    int* x = er_malloc(len, &p3);

    int og_data[7];
    for (int i = 0; i < 7; i++) {
        og_data[i] = i;
    }

    er_write_buf(x, &og_data, 0, len);

    // this will yield encrypted values
    for (int i = 0; i < 7; i++) {
        printf("x[%d] = %d\n", i, x[i]);
    }

    x[3] ^= 1 << 30;

    int recv[7];
    er_read_buf(x, &recv, 0, len);
    
    for (int i = 0; i < 7; i++) {
        printf("recv[%d] = %d\n", i, recv[i]);
    }

    x[0] ^= 1 << 5;

    int recv2[7];
    er_read_buf(x, &recv2, 0, len);
    
    for (int i = 0; i < 7; i++) {
        printf("recv2[%d] = %d\n", i, recv2[i]);
    }

    int x3[7];
    for (int i = 0; i < 7; i++) {
        x3[i] = i+10;
    }

    x[0] ^= 1 << 5;

    er_write_buf(x, &x3, 0, len);
    for (int i = 0; i < 7; i++) {
        printf("x[%d] = %d\n", i, x[i]);
    }

    x[4] ^= 1 << 5;

    int recv3[3];
    er_read_buf(x, &recv3, 2*sizeof(int), 3*sizeof(int));
    
    for (int i = 0; i < 3; i++) {
        printf("recv3[%d] = %d\n", i, recv3[i]);
    }

    x[4] ^= 1 << 5;

    int x4[3];
    for (int i = 0; i < 3; i++) {
        x4[i] = i+20;
    }
    er_write_buf(x, &x4, 3*sizeof(int), 3*sizeof(int));


    int recv4[7];
    er_read_buf(x, &recv4, 0, len);
    
    for (int i = 0; i < 7; i++) {
        printf("recv4[%d] = %d\n", i, recv4[i]);
    }

    END_FUNC;

}

void resilience_test(void) {
    START_FUNC;

    // This list of policies will need to be reordered 
    // into Redundancy -> ReedSol -> Encrypted
    struct er_policy_list p = {
        .policy = Encrypted,
        .policy_data = NULL,
        .next_policy = NULL
    };

    struct er_policy_list p2 = {
        .policy = Redundancy,
        .policy_data = &(int){3},
        .next_policy = &p
    };

    struct er_policy_list p3 = {
        .policy = ReedSolomon,
        .policy_data = &(int){3},
        .next_policy = &p2
    };

    size_t len = 5;

    char* og_data = "rise";

    char* x = er_malloc(len, &p3);
    er_write_buf(x, og_data, 0, len);

    // Multiple bit flips within a chunk
    // Need both FEC and Redundancy to correct
    x[0] ^= 1 << 3;
    x[2] ^= 1 << 2;
    x[3] ^= 1 << 2;
    x[3] ^= 1 << 3;
    x[3] ^= 1 << 7;
    
    char recv[5];
    int c = er_read_buf(x, &recv, 0, len);
    printf("num corrected errors: %d\n", c);

    printf("recv: %s\n", recv);
    for (int i = 0; i < len; i++) {
        printf("recv[%d] = %c\n", i, recv[i]);
    }

    END_FUNC;
}

int main(void)
{
    malloc_free_test();
    redundant_test();
    rs_test();
    rs_and_redundant_test();
    encryption_test();
    combined_test();
    resilience_test();
    return 0;
}
