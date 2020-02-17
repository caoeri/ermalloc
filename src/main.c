#include <stdlib.h>
#include <stdio.h>

#include "ermalloc.h"

int main(void)
{
    er_free(NULL);
    printf("hey\n");
    return 0;
}
