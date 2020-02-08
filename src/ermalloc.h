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

// The following functions behave the same as the original, no policies
void* malloc(size_t size);
void  free(void* ptr);
void* calloc(size_t nmemb, size_t size);
void* realloc(void* ptr, size_t size);
void* reallocarray(void* ptr, size_t nmemb, size_t size);

/**
 * Allocate uninitialized memory
 *
 * @param policies policies for the region, NULL for no policies
 */
void* ermalloc(size_t size, struct ermalloc_policy_list* policies);

/**
 * Same as free
 */
void  erfree(void* ptr);

/**
 * Allocate memory and zero it out
 *
 * @param policies policies for the region, NULL for no policies
 */
void* ercalloc(size_t nmemb, size_t size, struct ermalloc_policy_list* policies);

/**
 * Reallocate and resize a block of memory
 *
 * @param policies The policies to apply to the newly allocated block
 * Any original policies will be used to maintain data integrity while moving the allocation
 */
void* errealloc(void* ptr, size_t size, struct ermalloc_policy_list* policies);

/**
 * Reallocate and resize a block of memory
 *
 * @param policies The policies to apply to the newly allocated block
 * Any original policies will be used to maintain data integrity while moving the allocation
 */
void* erreallocarray(void* ptr, size_t nmemb, size_t size, struct ermalloc_policy_list* policies);

