#include <stdlib.h>
#include <stdio.h>

#include "ermalloc.h"

int main(void)
{
    int* x = er_malloc(123, NULL);
    x[12] = 7;
    er_free(x);
    return 0;
}
