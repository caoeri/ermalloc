#include <stdlib.h>
#include <stdio.h>

#include "ermalloc.h"

int main(void)
{
    int* x = er_malloc(8, NULL);
    x[0] = 1;
    x[1] = 4;
    int sum = x[0] + x[1];
    printf("The sum is %d\n", sum);
    er_free(x);
    return 0;
}
