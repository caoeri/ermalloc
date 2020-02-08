#include <stdlib.h>

#define MAX_POLICIES (3)

enum ermalloc_policy {
    Nil = 0,
    Redundancy
};

struct ermalloc_policy_list {
    enum ermalloc_policy policy;
    void* policy_data;
    struct ermalloc_policy_list* next_policy;
};

void* malloc(size_t size);
void  free(void* ptr);
void* calloc(size_t nmemb, size_t size);
void* realloc(void* ptr, size_t size);
void* reallocarray(void* ptr, size_t nmemb, size_t size);

