#ifndef __TOL_HELPER_H__
#define __TOL_HELPER_H__

#define DEFINE_TOL_ARRAY_STRUCT(type) \
    typedef struct TOL_Array_##type { \
        type *data; \
        size_t len; \
    } TOL_Array_##type;

#endif // !__TOL_HELPER_H__
