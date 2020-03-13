#include <stdlib.h>

#define MAX_POLICIES (3)

enum er_policy {
    Nil = 0,
    Redundancy
};

struct er_policy_list {
    enum er_policy policy;
    const void* policy_data;
    const struct er_policy_list* next_policy;
};
/*
// The following functions behave the same as the original, no policies
void* malloc(size_t size);
void  free(void* ptr);
void* calloc(size_t nmemb, size_t size);
void* realloc(void* ptr, size_t size);
void* reallocarray(void* ptr, size_t nmemb, size_t size);
*/
/**
 * Allocate uninitialized memory
 *
 * @param policies policies for the region, NULL for no policies
 */
void* er_malloc(size_t size, const struct er_policy_list* policies);

/**
 * Same as free
 */
void  er_free(void* ptr);

/**
 * Allocate memory and zero it out
 *
 * @param policies policies for the region, NULL for no policies
 */
void* er_calloc(size_t nmemb, size_t size, const struct er_policy_list* policies);

/**
 * Reallocate and resize a block of memory
 *
 * @param policies The policies to apply to the newly allocated block
 * Any original policies will be used to maintain data integrity while moving the allocation
 */
void* er_realloc(void* ptr, size_t size, const struct er_policy_list* policies);

/**
 * Reallocate and resize a block of memory
 *
 * @param policies The policies to apply to the newly allocated block
 * Any original policies will be used to maintain data integrity while moving the allocation
 */
void* er_reallocarray(void* ptr, size_t nmemb, size_t size, const struct er_policy_list* policies);

/**
 * Change policies for an allocated region
 */
void er_change_policies(void* ptr, const struct er_policy_list* policies);

/**
 * After allocating a region with policies:
 * 1. Write your data into your buffer
 * 2. Call er_setup_policies to initialize the policies on your data
 * 3. Further calls to er_read/write_buf will correct this bits
 * 4. You can also use er_correct_buffer to manually apply corrections
 */
void er_setup_policies(const void* ptr);

/**
 * Use policies to find bit errors and correct them if possible and desired
 *
 * @return = 0 if no errors
 *         < 0 if unrecoverable errors, as defined by the associated policies
 *         > 0 number of errors found/corrected, as defined by the associated policies
 */
int er_correct_buffer(void* ptr);

/**
 * Enforce the policy and read the data
 * Depending on the policies selected,
 * the policy enforcement may act on the
 * entire allocated block, rather than just the desired region
 *
 * @param base Pointer to start of allocation
 * @param dest Pointer to destination buffer
 * @param offset Bytes after base to start reading from
 * @param len Number of bytes to read
 * @return = 0 if no errors
 *         < 0 if unrecoverable errors, as defined by the associated policies
 *         > 0 number of errors found/corrected, as defined by the associated policies
 */
int er_read_buf(void* base, void* dest, size_t offset, size_t len);

/**
 * Write the data and then enforce the policy on new data
 *
 * @param base Pointer to start of allocation
 * @param src  Pointer to start of read data
 * @param offset Bytes after base to start reading to
 * @param len Number of bytes to write
 * @return = 0 if no errors
 *         < 0 if unrecoverable errors, as defined by the associated policies
 *         > 0 number of errors found/corrected, as defined by the associated policies
 */
int er_write_buf(void* base, const void* src, size_t offset, size_t len);

