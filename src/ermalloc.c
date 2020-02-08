#include <stdlib.h>
#include "ermalloc.h"


void* malloc(size_t size)
{
    return er_malloc(size, NULL);
}

void  free(void* ptr)
{
    er_free(ptr);
}

void* calloc(size_t nmemb, size_t size)
{
    return er_calloc(nmemb, size, NULL);
}

void* realloc(void* ptr, size_t size)
{
    return er_realloc(ptr, size, NULL);
}

void* reallocarray(void* ptr, size_t nmemb, size_t size)
{
    return er_reallocarray(ptr, nmemb, size, NULL);
}

