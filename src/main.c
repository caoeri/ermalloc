#include <stdlib.h>
#include <stdio.h>

#include "ermalloc.h"

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
    return 0;
}
