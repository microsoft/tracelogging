// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 199309L
#endif

#include <lttngh/LttngHelpers.h>
#include <byteswap.h>
#include <lttng/ringbuffer-config.h> // lttng_ust_lib_ring_buffer_ctx
#include <urcu/compiler.h>           // caa_unlikely
#include <urcu/system.h>             // CMM_LOAD_SHARED

#define lttngh_RCU_DEREFERENCE(p) URCU_FORCE_CAST( \
    __typeof__(p),                                 \
    tp_rcu_dereference_sym_bp(URCU_FORCE_CAST(void *, p)))

extern int tracepoint_register_lib(struct lttng_ust_tracepoint *const *, int);
extern int tracepoint_unregister_lib(struct lttng_ust_tracepoint *const *);
extern void tp_rcu_read_lock_bp(void);
extern void tp_rcu_read_unlock_bp(void);
extern void *tp_rcu_dereference_sym_bp(void *);

#ifdef USE_LTTNG_2_7
#define EnumEntryUnsigned(n, name) \
    {                              \
        (n), (n), name, {}         \
    }
#else
#define EnumEntryUnsigned(n, name)   \
    {                                \
        {(n), 0}, {(n), 0}, name, {} \
    }
#endif

static struct lttng_enum_entry const BoolEnumEntries[] = {
    EnumEntryUnsigned(0, "false"),
    EnumEntryUnsigned(1, "true")};

#ifdef USE_LTTNG_2_7
struct lttng_enum const lttngh_BoolEnumDesc = {
    "bool", {atype_enum, {}}, BoolEnumEntries, 2, {}};
#else
struct lttng_enum_desc const lttngh_BoolEnumDesc = {
    "bool", BoolEnumEntries, 2, {}};
#endif

static unsigned Utf16ToUtf8Size(
    char16_t const *pch16,
    unsigned cch16)
    lttng_ust_notrace;
static unsigned Utf16ToUtf8Size(
    char16_t const *pch16,
    unsigned cch16)
{
    unsigned ich8 = 0;
    unsigned ich16 = 0;

    // Since we never get cch16 > 65535, we can safely skip testing for overflow of ich8.
    assert(cch16 <= (0xFFFFFFFF / 3));

    while (ich16 != cch16)
    {
        // Note that this algorithm accepts unmatched surrogate pairs.
        // That's probably the right decision for logging - we want to preserve
        // them so they can be noticed and fixed.
        unsigned val16 = pch16[ich16];
        ich16 += 1;
        if (caa_likely(val16 < 0x80))
        {
            ich8 += 1;
        }
        else if (caa_likely(val16 < 0x800))
        {
            ich8 += 2;
        }
        else if (
            0xd800 <= val16 && val16 < 0xdc00 &&
            ich16 != cch16 &&
            0xdc00 <= pch16[ich16] && pch16[ich16] < 0xe000)
        {
            // Valid surrogate pair.
            ich16 += 1;
            ich8 += 4;
        }
        else
        {
            ich8 += 3;
        }
    }
    return ich8;
}

static unsigned Utf16ToUtf8(
    char16_t const *pch16,
    unsigned cch16,
    unsigned char *pch8,
    unsigned cch8)
    lttng_ust_notrace;
static unsigned Utf16ToUtf8(
    char16_t const *pch16,
    unsigned cch16,
    unsigned char *pch8,
    unsigned cch8)
{
    unsigned ich8 = 0;
    unsigned ich16 = 0;

    // Since we never get cch16 > 65535, we can safely skip testing for overflow of ich8.
    assert(cch16 <= (0xFFFFFFFF / 3));

    while (ich16 != cch16)
    {
        // Note that this algorithm accepts unmatched surrogate pairs.
        // That's probably the right decision for logging - we want to preserve
        // them so they can be noticed and fixed.
        unsigned val16 = pch16[ich16];
        ich16 += 1;
        if (caa_likely(val16 < 0x80))
        {
            if (caa_unlikely(ich8 == cch8))
                break;
            pch8[ich8++] = (unsigned char)val16;
        }
        else if (caa_likely(val16 < 0x800))
        {
            if (caa_unlikely(ich8 + 1 >= cch8))
                break;
            pch8[ich8++] = (unsigned char)(((val16 >> 6)) | 0xc0);
            pch8[ich8++] = (unsigned char)(((val16)&0x3f) | 0x80);
        }
        else if (
            0xd800 <= val16 && val16 < 0xdc00 &&
            ich16 != cch16 &&
            0xdc00 <= pch16[ich16] && pch16[ich16] < 0xe000)
        {
            // Valid surrogate pair.
            if (caa_unlikely(ich8 + 3 >= cch8))
                break;
            val16 = 0x010000u + (((val16 - 0xd800u) << 10) | (pch16[ich16] - 0xdc00u));
            ich16 += 1;
            pch8[ich8++] = (unsigned char)(((val16 >> 18)) | 0xf0);
            pch8[ich8++] = (unsigned char)(((val16 >> 12) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val16 >> 6) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val16)&0x3f) | 0x80);
        }
        else
        {
            if (caa_unlikely(ich8 + 2 >= cch8))
                break;
            pch8[ich8++] = (unsigned char)(((val16 >> 12)) | 0xe0);
            pch8[ich8++] = (unsigned char)(((val16 >> 6) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val16)&0x3f) | 0x80);
        }
    }

    return ich8;
}

static unsigned Utf32ToUtf8Size(
    char32_t const *pch32,
    unsigned cch32)
    lttng_ust_notrace;
static unsigned Utf32ToUtf8Size(
    char32_t const *pch32,
    unsigned cch32)
{
    unsigned ich8 = 0;
    unsigned ich32 = 0;

    // Since we never get cch32 > 65535, we can safely skip testing for overflow of ich8.
    assert(cch32 <= (0xFFFFFFFF / 7));

    while (ich32 != cch32)
    {
        // Note that this algorithm accepts non-Unicode values (above 0x10FFFF).
        // That's probably the right decision for logging - we want to preserve
        // them so they can be noticed and fixed.
        unsigned val32 = pch32[ich32];
        ich32 += 1;
        if (caa_likely(val32 < 0x80))
        {
            ich8 += 1;
        }
        else if (caa_likely(val32 < 0x800))
        {
            ich8 += 2;
        }
        else if (caa_likely(val32 < 0x10000))
        {
            ich8 += 3;
        }
        else if (caa_likely(val32 < 0x200000))
        {
            ich8 += 4;
        }
        else if (caa_likely(val32 < 0x4000000))
        {
            ich8 += 5;
        }
        else if (caa_likely(val32 < 0x80000000))
        {
            ich8 += 6;
        }
        else
        {
            ich8 += 7;
        }
    }
    return ich8;
}

static unsigned Utf32ToUtf8(
    char32_t const *pch32,
    unsigned cch32,
    unsigned char *pch8,
    unsigned cch8)
    lttng_ust_notrace;
static unsigned Utf32ToUtf8(
    char32_t const *pch32,
    unsigned cch32,
    unsigned char *pch8,
    unsigned cch8)
{
    unsigned ich8 = 0;
    unsigned ich32 = 0;

    // Since we never get cch32 > 65535, we can safely skip testing for overflow of ich8.
    assert(cch32 <= (0xFFFFFFFF / 7));

    while (ich32 != cch32)
    {
        // Note that this algorithm accepts unmatched surrogate pairs.
        // That's probably the right decision for logging - we want to preserve
        // them so they can be noticed and fixed.
        unsigned val32 = pch32[ich32];
        ich32 += 1;
        if (caa_likely(val32 < 0x80))
        {
            if (caa_unlikely(ich8 == cch8))
                break;
            pch8[ich8++] = (unsigned char)val32;
        }
        else if (caa_likely(val32 < 0x800))
        {
            if (caa_unlikely(ich8 + 1 >= cch8))
                break;
            pch8[ich8++] = (unsigned char)(((val32 >> 6)) | 0xc0);
            pch8[ich8++] = (unsigned char)(((val32)&0x3f) | 0x80);
        }
        else if (caa_likely(val32 < 0x10000))
        {
            if (caa_unlikely(ich8 + 2 >= cch8))
                break;
            pch8[ich8++] = (unsigned char)(((val32 >> 12)) | 0xe0);
            pch8[ich8++] = (unsigned char)(((val32 >> 6) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32)&0x3f) | 0x80);
        }
        else if (caa_likely(val32 < 0x200000))
        {
            if (caa_unlikely(ich8 + 3 >= cch8))
                break;
            pch8[ich8++] = (unsigned char)(((val32 >> 18)) | 0xf0);
            pch8[ich8++] = (unsigned char)(((val32 >> 12) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32 >> 6) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32)&0x3f) | 0x80);
        }
        else if (caa_likely(val32 < 0x4000000))
        {
            if (caa_unlikely(ich8 + 4 >= cch8))
                break;
            pch8[ich8++] = (unsigned char)(((val32 >> 24)) | 0xf8);
            pch8[ich8++] = (unsigned char)(((val32 >> 18) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32 >> 12) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32 >> 6) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32)&0x3f) | 0x80);
        }
        else if (caa_likely(val32 < 0x80000000))
        {
            if (caa_unlikely(ich8 + 5 >= cch8))
                break;
            pch8[ich8++] = (unsigned char)(((val32 >> 30)) | 0xfc);
            pch8[ich8++] = (unsigned char)(((val32 >> 24) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32 >> 18) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32 >> 12) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32 >> 6) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32)&0x3f) | 0x80);
        }
        else
        {
            if (caa_unlikely(ich8 + 6 >= cch8))
                break;
            pch8[ich8++] = (unsigned char)0xfe;
            pch8[ich8++] = (unsigned char)(((val32 >> 30) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32 >> 24) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32 >> 18) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32 >> 12) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32 >> 6) & 0x3f) | 0x80);
            pch8[ich8++] = (unsigned char)(((val32)&0x3f) | 0x80);
        }
    }

    return ich8;
}

static void ProviderError(
    struct lttng_probe_desc *pProbeDesc,
    int err,
    const char *msg)
    __attribute__((noreturn)) lttng_ust_notrace;
static void ProviderError(
    struct lttng_probe_desc *pProbeDesc,
    int err,
    const char *msg)
{
    fprintf(stderr, "LTTng-UST: provider \"%s\" error %d: %s\n",
            pProbeDesc->provider, err, msg);
    abort();
}

static int FixArrayCompare(
    void const *p1, void const *p2)
    lttng_ust_notrace;
static int FixArrayCompare(
    void const *p1, void const *p2)
{
    // Reverse sort so that NULL goes at end.
    void const *v1 = *(void const *const *)p1;
    void const *v2 = *(void const *const *)p2;
    return v1 < v2 ? 1 : v1 == v2 ? 0 : -1;
}

// Remove duplicates and NULLs from an array of pointers.
static void *FixArray(
    void const **ppStart,
    void const **ppEnd)
    lttng_ust_notrace;
static void *FixArray(
    void const **ppStart,
    void const **ppEnd)
{
    void const **ppGood = ppStart;

    if (ppStart != ppEnd)
    {
        // Sort.
        qsort(ppStart, (size_t)(ppEnd - ppStart), sizeof(void *), FixArrayCompare);

        // Remove adjacent repeated elements.
        for (; ppGood + 1 != ppEnd; ppGood += 1)
        {
            if (*ppGood == *(ppGood + 1))
            {
                for (void const **ppNext = ppGood + 2; ppNext != ppEnd; ppNext += 1)
                {
                    if (*ppGood != *ppNext)
                    {
                        ppGood += 1;
                        *ppGood = *ppNext;
                    }
                }
                break;
            }
        }

        if (*ppGood != NULL)
        {
            ppGood += 1;
        }
    }

    return ppGood;
}

// ust-tracepoint-event.h(991) __lttng_events_init
int lttngh_RegisterProvider(
    int volatile *pIsRegistered,
    struct lttng_probe_desc *pProbeDesc,
    struct lttng_ust_tracepoint **pTracepointStart,
    struct lttng_ust_tracepoint **pTracepointStop,
    struct lttng_event_desc const **pEventDescStart,
    struct lttng_event_desc const **pEventDescStop)
{
    int err;

    // Note: not intended to support multithreaded or ref-counted registration.
    // pIsRegistered is used to make it safe to call Unregister on an
    // unregistered lib and to help detect misuse, not to make this thread-safe.
    if (__atomic_exchange_n(pIsRegistered, 1, __ATOMIC_RELAXED) != 0)
    {
        err = EEXIST;
        ProviderError(pProbeDesc, err,
                      "provider already registered.");
    }
    else
    {
        struct lttng_event_desc const **const pEventDescLast =
            (struct lttng_event_desc const **)FixArray(
                (void const **)pEventDescStart, (void const **)pEventDescStop);
        pProbeDesc->event_desc = pEventDescStart;
        pProbeDesc->nr_events = (unsigned)(pEventDescLast - pEventDescStart);
        err = lttng_probe_register(pProbeDesc);
        if (err != 0)
        {
            ProviderError(pProbeDesc, err,
                          "lttng_probe_register failed."
                          " (Registration of multiple providers having the same name is not supported.)");
            __atomic_exchange_n(pIsRegistered, 0, __ATOMIC_RELAXED);
        }
        else
        {
            struct lttng_ust_tracepoint *const *const pTracepointLast =
                (struct lttng_ust_tracepoint *const *)FixArray(
                    (void const **)pTracepointStart, (void const **)pTracepointStop);

            // May fail for out-of-memory. Continue anyway.
            err = tracepoint_register_lib(pTracepointStart, (int)(pTracepointLast - pTracepointStart));
            if (err != 0)
            {
                lttng_probe_unregister(pProbeDesc);
                __atomic_exchange_n(pIsRegistered, 0, __ATOMIC_RELAXED);
            }
        }
    }

    return err;
}

// ust-tracepoint-event.h(1018) __lttng_events_exit
int lttngh_UnregisterProvider(
    int volatile *pIsRegistered,
    struct lttng_probe_desc *pProbeDesc,
    struct lttng_ust_tracepoint *const *pTracepointStart)
{
    int err = 0;

    // Calling Unregister on an unregistered lib is a safe no-op as long as it
    // doesn't race with a Register (which is a bug in the caller).
    if (__atomic_exchange_n(pIsRegistered, 0, __ATOMIC_RELAXED) != 0)
    {
        err = tracepoint_unregister_lib(pTracepointStart);
        lttng_probe_unregister(pProbeDesc);
    }

    return err;
}

static int EventProbeFilter(
    struct lttng_event const *pEvent,
    struct lttngh_DataDesc const *pDataDesc,
    unsigned cDataDesc)
    lttng_ust_notrace;
static int EventProbeFilter(
    struct lttng_event const *pEvent,
    struct lttngh_DataDesc const *pDataDesc,
    unsigned cDataDesc)
{
    int filterRecord = pEvent->has_enablers_without_bytecode;
    unsigned cbBuffer;

    cbBuffer = 0;
    for (unsigned i = 0; i != cDataDesc; i += 1)
    {
        switch (pDataDesc[i].Type)
        {
        case lttngh_DataType_None:
            break;
        case lttngh_DataType_SignedLE:
        case lttngh_DataType_SignedBE:
            cbBuffer += (unsigned)sizeof(int64_t);
            break;
        case lttngh_DataType_UnsignedLE:
        case lttngh_DataType_UnsignedBE:
            cbBuffer += (unsigned)sizeof(uint64_t);
            break;
        case lttngh_DataType_FloatLE:
        case lttngh_DataType_FloatBE:
            cbBuffer += (unsigned)sizeof(double);
            break;
        case lttngh_DataType_String8:
        case lttngh_DataType_StringUtf16Transcoded:
        case lttngh_DataType_StringUtf32Transcoded:
            cbBuffer += (unsigned)sizeof(void *);
            break;
        case lttngh_DataType_Counted:
        case lttngh_DataType_SequenceUtf16Transcoded:
        case lttngh_DataType_SequenceUtf32Transcoded:
            cbBuffer += (unsigned)(sizeof(unsigned long) + sizeof(void *));
            break;
        default:
            abort();
            break;
        }
    }

    // Scope for VLA
    {
        char stackData[cbBuffer] __attribute__((aligned(sizeof(size_t)))); // VLA
        char *pStackData = stackData;

        for (unsigned i = 0; i != cDataDesc; i += 1)
        {
            switch (pDataDesc[i].Type)
            {
            case lttngh_DataType_None:
                break;
            case lttngh_DataType_Signed:
            {
                switch (pDataDesc[i].Size)
                {
                case 1:
                {
                    int64_t val;
                    val = *(int8_t const *)pDataDesc[i].Data; // Sign-extend
                    memcpy(pStackData, &val, sizeof(int64_t));
                    break;
                }
                case 2:
                {
                    int64_t val;
                    int16_t tempVal;
                    memcpy(&tempVal, pDataDesc[i].Data, sizeof(tempVal));
                    val = tempVal; // Sign-extend
                    memcpy(pStackData, &val, sizeof(int64_t));
                    break;
                }
                case 4:
                {
                    int64_t val;
                    int32_t tempVal;
                    memcpy(&tempVal, pDataDesc[i].Data, sizeof(tempVal));
                    val = tempVal; // Sign-extend
                    memcpy(pStackData, &val, sizeof(int64_t));
                    break;
                }
                case 8:
                {
                    memcpy(pStackData, pDataDesc[i].Data, sizeof(int64_t));
                    break;
                }
                default:
                    abort();
                    break;
                }
                pStackData += sizeof(int64_t);
                break;
            }
            case (lttngh_DataType_Signed == lttngh_DataType_SignedLE
                ? lttngh_DataType_SignedBE
                : lttngh_DataType_SignedLE):
                {
                    switch (pDataDesc[i].Size)
                    {
                    case 1:
                    {
                        int64_t val;
                        val = *(int8_t const *)pDataDesc[i].Data; // Sign-extend
                        memcpy(pStackData, &val, sizeof(int64_t));
                        break;
                    }
                    case 2:
                    {
                        int64_t val;
                        int16_t tempVal;
                        uint16_t tempValSwap;
                        memcpy(&tempValSwap, pDataDesc[i].Data, sizeof(tempValSwap));
                        tempValSwap = bswap_16(tempValSwap);
                        memcpy(&tempVal, &tempValSwap, sizeof(tempVal));
                        val = tempVal; // Sign-extend
                        memcpy(pStackData, &val, sizeof(int64_t));
                        break;
                    }
                    case 4:
                    {
                        int64_t val;
                        int32_t tempVal;
                        uint32_t tempValSwap;
                        memcpy(&tempValSwap, pDataDesc[i].Data, sizeof(tempValSwap));
                        tempValSwap = bswap_32(tempValSwap);
                        memcpy(&tempVal, &tempValSwap, sizeof(tempVal));
                        val = tempVal; // Sign-extend
                        memcpy(pStackData, &val, sizeof(int64_t));
                        break;
                    }
                    case 8:
                    {
                        uint64_t tempValSwap;
                        memcpy(&tempValSwap, pDataDesc[i].Data, sizeof(tempValSwap));
                        tempValSwap = bswap_64(tempValSwap);
                        memcpy(pStackData, &tempValSwap, sizeof(int64_t));
                        break;
                    }
                    default:
                        abort();
                        break;
                    }
                    pStackData += sizeof(int64_t);
                    break;
                }
            case lttngh_DataType_Unsigned:
            {
                switch (pDataDesc[i].Size)
                {
                case 1:
                {
                    uint64_t val;
                    val = *(uint8_t const *)pDataDesc[i].Data; // Zero-extend
                    memcpy(pStackData, &val, sizeof(uint64_t));
                    break;
                }
                case 2:
                {
                    uint64_t val;
                    uint16_t tempVal;
                    memcpy(&tempVal, pDataDesc[i].Data, sizeof(tempVal));
                    val = tempVal; // Zero-extend
                    memcpy(pStackData, &val, sizeof(uint64_t));
                    break;
                }
                case 4:
                {
                    uint64_t val;
                    uint32_t tempVal;
                    memcpy(&tempVal, pDataDesc[i].Data, sizeof(tempVal));
                    val = tempVal; // Zero-extend
                    memcpy(pStackData, &val, sizeof(uint64_t));
                    break;
                }
                case 8:
                {
                    memcpy(pStackData, pDataDesc[i].Data, sizeof(uint64_t));
                    break;
                }
                default:
                    abort();
                    break;
                }
                pStackData += sizeof(uint64_t);
                break;
            }
            case (lttngh_DataType_Unsigned == lttngh_DataType_UnsignedLE
                ? lttngh_DataType_UnsignedBE
                : lttngh_DataType_UnsignedLE):
                {
                    switch (pDataDesc[i].Size)
                    {
                    case 1:
                    {
                        uint64_t val;
                        val = *(uint8_t const *)pDataDesc[i].Data; // Zero-extend
                        memcpy(pStackData, &val, sizeof(uint64_t));
                        break;
                    }
                    case 2:
                    {
                        uint64_t val;
                        uint16_t tempValSwap;
                        memcpy(&tempValSwap, pDataDesc[i].Data, sizeof(tempValSwap));
                        tempValSwap = bswap_16(tempValSwap);
                        val = tempValSwap; // Zero-extend
                        memcpy(pStackData, &val, sizeof(uint64_t));
                        break;
                    }
                    case 4:
                    {
                        uint64_t val;
                        uint32_t tempValSwap;
                        memcpy(&tempValSwap, pDataDesc[i].Data, sizeof(tempValSwap));
                        tempValSwap = bswap_32(tempValSwap);
                        val = tempValSwap; // Zero-extend
                        memcpy(pStackData, &val, sizeof(uint64_t));
                        break;
                    }
                    case 8:
                    {
                        uint64_t tempValSwap;
                        memcpy(&tempValSwap, pDataDesc[i].Data, sizeof(tempValSwap));
                        tempValSwap = bswap_64(tempValSwap);
                        memcpy(pStackData, &tempValSwap, sizeof(uint64_t));
                        break;
                    }
                    default:
                        abort();
                        break;
                    }
                    pStackData += sizeof(uint64_t);
                    break;
                }
            case lttngh_DataType_Float:
            {
                if (pDataDesc[i].Size == sizeof(float))
                {
                    double val;
                    float tempVal;
                    memcpy(&tempVal, pDataDesc[i].Data, sizeof(tempVal));
                    val = tempVal; // Convert
                    memcpy(pStackData, &val, sizeof(double));
                }
                else if (pDataDesc[i].Size == sizeof(double))
                {
                    memcpy(pStackData, pDataDesc[i].Data, sizeof(double));
                }
                else if (pDataDesc[i].Size == sizeof(long double))
                {
                    double val;
                    long double tempVal;
                    memcpy(&tempVal, pDataDesc[i].Data, sizeof(tempVal));
                    val = (double)tempVal; // Convert
                    memcpy(pStackData, &val, sizeof(double));
                }
                else
                {
                    abort();
                }
                pStackData += sizeof(double);
                break;
            }
            case (lttngh_DataType_Float == lttngh_DataType_FloatLE
                ? lttngh_DataType_FloatBE
                : lttngh_DataType_FloatLE):
                {
                    if (pDataDesc[i].Size == sizeof(float))
                    {
                        double val;
                        float tempVal;
                        uint32_t tempValSwap;
                        memcpy(&tempValSwap, pDataDesc[i].Data, sizeof(tempValSwap));
                        tempValSwap = bswap_32(tempValSwap);
                        memcpy(&tempVal, &tempValSwap, sizeof(tempVal));
                        val = tempVal; // Convert
                        memcpy(pStackData, &val, sizeof(double));
                    }
                    else if (pDataDesc[i].Size == sizeof(double))
                    {
                        uint64_t tempValSwap;
                        memcpy(&tempValSwap, pDataDesc[i].Data, sizeof(tempValSwap));
                        tempValSwap = bswap_64(tempValSwap);
                        memcpy(pStackData, &tempValSwap, sizeof(double));
                    }
                    else if (pDataDesc[i].Size == sizeof(long double))
                    {
                        double val;
                        long double tempVal;
                        char tempValSwap[sizeof(long double)];
                        char const *p = pDataDesc[i].Data;
                        unsigned iSwap = sizeof(long double);
                        do
                        {
                            iSwap -= 1;
                            tempValSwap[iSwap] = *p;
                            p += 1;
                        } while (iSwap != 0);
                        memcpy(&tempVal, &tempValSwap, sizeof(tempVal));
                        val = (double)tempVal; // Convert
                        memcpy(pStackData, &val, sizeof(double));
                    }
                    else
                    {
                        abort();
                    }
                    pStackData += sizeof(double);
                    break;
                }
            case lttngh_DataType_String8:
            case lttngh_DataType_StringUtf16Transcoded:
            case lttngh_DataType_StringUtf32Transcoded:
            {
                // TODO - convert to utf8 for filtering?
                memcpy(pStackData, &pDataDesc[i].Data, sizeof(void *));
                pStackData += sizeof(void *);
                break;
            }
            case lttngh_DataType_Counted:
            {
                unsigned long len = pDataDesc[i].Length;
                memcpy(pStackData, &len, sizeof(unsigned long));
                pStackData += sizeof(unsigned long);
                memcpy(pStackData, &pDataDesc[i].Data, sizeof(void *));
                pStackData += sizeof(void *);
                break;
            }
            case lttngh_DataType_SequenceUtf16Transcoded:
            case lttngh_DataType_SequenceUtf32Transcoded:
            {
                // TODO - convert to utf8 for filtering?
                unsigned long len = pDataDesc[i].Size; // Type says "count of utf-8 bytes", so use size.
                memcpy(pStackData, &len, sizeof(unsigned long));
                pStackData += sizeof(unsigned long);
                memcpy(pStackData, &pDataDesc[i].Data, sizeof(void *));
                pStackData += sizeof(void *);
                break;
            }
            default:
                abort();
                break;
            }
        }

        for (struct lttng_bytecode_runtime *pRuntime = cds_list_entry(
                 lttngh_RCU_DEREFERENCE(pEvent->bytecode_runtime_head.next), __typeof__(*pRuntime), node);
             &pRuntime->node != &pEvent->bytecode_runtime_head;
             pRuntime = cds_list_entry(
                 lttngh_RCU_DEREFERENCE(pRuntime->node.next), __typeof__(*pRuntime), node))
        {
            if (caa_unlikely(pRuntime->filter(pRuntime, stackData) & LTTNG_FILTER_RECORD_FLAG))
            {
                filterRecord = 1;
            }
        }
    }

    return filterRecord;
}

int lttngh_EventProbe(
    struct lttng_ust_tracepoint *pTracepoint,
    struct lttngh_DataDesc *pDataDesc,
    unsigned cDataDesc,
    void *pCallerIp)
{
    int err = 0;
    struct lttng_ust_tracepoint_probe *pTpProbe;

    unsigned maxAlign = 0; // 0 means we haven't computed event size yet.
    unsigned cbData = 0;
    unsigned char transcodeScratchOnStack[256] __attribute__((aligned(2)));
    unsigned char *pbTranscodeScratch = transcodeScratchOnStack;
    unsigned cbTranscodeScratch = sizeof(transcodeScratchOnStack);
    unsigned cbTranscodeWanted = 0;

    if (pCallerIp == NULL)
    {
        pCallerIp = lttngh_IP_PARAM;
    }

    tp_rcu_read_lock_bp();
    pTpProbe = lttngh_RCU_DEREFERENCE(pTracepoint->probes);
    if (caa_likely(pTpProbe))
    {
        do
        {
            struct lttng_event *const pEvent = (struct lttng_event *const)pTpProbe->data;
            struct lttng_channel const *const pChannel = pEvent->chan;

            if (caa_likely(CMM_ACCESS_ONCE(pChannel->session->active)) &&
                caa_likely(CMM_ACCESS_ONCE(pChannel->enabled)) &&
                caa_likely(CMM_ACCESS_ONCE(pEvent->enabled)) &&
#ifdef TP_SESSION_CHECK // Are we building statedump?
                session == pChannel->session &&
#endif // TP_SESSION_CHECK
                (caa_likely(cds_list_empty(&pEvent->bytecode_runtime_head)) ||
                 caa_unlikely(EventProbeFilter(pEvent, pDataDesc, cDataDesc))))
            {
                // Compute event size the first time through.
                if (caa_likely(maxAlign == 0))
                {
                    maxAlign = 1;
                    assert(cbData == 0);

                    for (unsigned i = 0; i != cDataDesc; i += 1)
                    {
                        switch (pDataDesc[i].Type)
                        {
                        case lttngh_DataType_StringUtf16Transcoded:
                        case lttngh_DataType_StringUtf32Transcoded:
                        {
                            unsigned cbUtf8;

                            if (pDataDesc[i].Type == lttngh_DataType_StringUtf16Transcoded)
                            {
                                unsigned cch16 = pDataDesc[i].Size / sizeof(char16_t);
                                assert(cch16 != 0); // Error in caller - Size should have included NUL.
                                cch16 -= 1;         // Do not count NUL.

                                if (caa_unlikely(cch16 > 65535))
                                {
                                    if (cch16 == ~0u)
                                    {
                                        cch16 = 0; // Error in caller - Size was 0.
                                    }
                                    else
                                    {
                                        cch16 = 65535;
                                    }
                                }

                                cbUtf8 = Utf16ToUtf8Size(pDataDesc[i].Data, cch16);
                            }
                            else
                            {
                                unsigned cch32 = pDataDesc[i].Size / sizeof(char32_t);
                                assert(cch32 != 0); // Error in caller - Size should have included NUL.
                                cch32 -= 1;         // Do not count NUL.

                                if (caa_unlikely(cch32 > 65535))
                                {
                                    if (cch32 == ~0u)
                                    {
                                        cch32 = 0; // Error in caller - Size was 0.
                                    }
                                    else
                                    {
                                        cch32 = 65535;
                                    }
                                }

                                cbUtf8 = Utf32ToUtf8Size(pDataDesc[i].Data, cch32);
                            }

                            if (caa_unlikely(cbUtf8 > 65535))
                            {
                                cbUtf8 = 65535;
                            }

                            // Use DataDesc.Length as scratch space to store utf-8 content size.
                            pDataDesc[i].Length = (uint16_t)cbUtf8; // Does not count NUL.

                            cbUtf8 += 1; // Include room for 8-bit NUL.
                            if (caa_unlikely(cbTranscodeWanted < cbUtf8))
                            {
                                cbTranscodeWanted = cbUtf8;
                            }

                            cbData += cbUtf8;
                            if (caa_unlikely(cbData < cbUtf8))
                            {
                                err = EOVERFLOW;
                                goto Done;
                            }
                            break;
                        }
                        case lttngh_DataType_SequenceUtf16Transcoded:
                        case lttngh_DataType_SequenceUtf32Transcoded:
                        {
#ifdef RING_BUFFER_ALIGN
                            if (__alignof__(uint16_t) > maxAlign)
                            {
                                maxAlign = __alignof__(uint16_t);
                            }

                            unsigned const alignAdjust =
                                lib_ring_buffer_align(cbData, __alignof__(uint16_t));
                            cbData += alignAdjust;
                            if (caa_unlikely(cbData < alignAdjust))
                            {
                                err = EOVERFLOW;
                                goto Done;
                            }
#endif // RING_BUFFER_ALIGN

                            unsigned cbUtf8;

                            if (pDataDesc[i].Type == lttngh_DataType_SequenceUtf16Transcoded)
                            {
                                unsigned cch16 = pDataDesc[i].Size / sizeof(char16_t);
                                if (caa_unlikely(cch16 > 65535))
                                {
                                    cch16 = 65535;
                                }

                                cbUtf8 = Utf16ToUtf8Size(pDataDesc[i].Data, cch16);
                            }
                            else
                            {
                                unsigned cch32 = pDataDesc[i].Size / sizeof(char32_t);
                                if (caa_unlikely(cch32 > 65535))
                                {
                                    cch32 = 65535;
                                }

                                cbUtf8 = Utf32ToUtf8Size(pDataDesc[i].Data, cch32);
                            }

                            if (caa_unlikely(cbUtf8 > 65535))
                            {
                                cbUtf8 = 65535;
                            }

                            // Use DataDesc.Length as scratch space to store utf-8 content size.
                            pDataDesc[i].Length = (uint16_t)cbUtf8;

                            cbUtf8 += 2; // Include room for 16-bit length.
                            if (caa_unlikely(cbTranscodeWanted < cbUtf8))
                            {
                                cbTranscodeWanted = cbUtf8;
                            }

                            cbData += cbUtf8;
                            if (caa_unlikely(cbData < cbUtf8))
                            {
                                err = EOVERFLOW;
                                goto Done;
                            }
                            break;
                        }
                        default:
                        {
#ifdef RING_BUFFER_ALIGN
                            if (pDataDesc[i].Alignment > maxAlign)
                            {
                                maxAlign = pDataDesc[i].Alignment;
                            }

                            unsigned const alignAdjust =
                                lib_ring_buffer_align(cbData, pDataDesc[i].Alignment);
                            cbData += alignAdjust;
                            if (caa_unlikely(cbData < alignAdjust))
                            {
                                err = EOVERFLOW;
                                goto Done;
                            }
#endif // RING_BUFFER_ALIGN

                            cbData += pDataDesc[i].Size;
                            if (cbData < pDataDesc[i].Size)
                            {
                                err = EOVERFLOW;
                                goto Done;
                            }
                            break;
                        }
                        }
                    }

                    // If our scratch buffer is too small, try to heap allocate.
                    if (caa_unlikely(cbTranscodeScratch < cbTranscodeWanted))
                    {
                        unsigned char *pbTranscodeWanted = (unsigned char *)malloc(cbTranscodeWanted);
                        if (caa_unlikely(pbTranscodeWanted == NULL))
                        {
                            err = ENOMEM;
                            goto Done;
                        }

                        cbTranscodeScratch = cbTranscodeWanted;
                        pbTranscodeScratch = pbTranscodeWanted;
                    }
                }

#ifndef USE_LTTNG_2_7
                struct lttng_stack_ctx stackContext = {0};
                stackContext.event = pEvent;
                stackContext.chan_ctx = lttngh_RCU_DEREFERENCE(pChannel->ctx);
                stackContext.event_ctx = lttngh_RCU_DEREFERENCE(pEvent->ctx);
#endif

                struct lttng_ust_lib_ring_buffer_ctx bufferContext;
                lib_ring_buffer_ctx_init(&bufferContext, pChannel->chan, pEvent,
                                         cbData, (int)maxAlign, -1, pChannel->handle
#ifndef USE_LTTNG_2_7
                                         ,
                                         &stackContext
#endif
                );
                bufferContext.ip = pCallerIp;

                int const reserveResult = pChannel->ops->event_reserve(&bufferContext, pEvent->id);
                if (caa_unlikely(reserveResult < 0))
                {
                    err = reserveResult;
                }
                else
                {
                    for (unsigned i = 0; i != cDataDesc; i += 1)
                    {
#ifdef RING_BUFFER_ALIGN
                        lib_ring_buffer_align_ctx(&bufferContext, pDataDesc[i].Alignment);
#endif // RING_BUFFER_ALIGN
                        switch (pDataDesc[i].Type)
                        {
                        case lttngh_DataType_String8:
                        {
                            if (pChannel->ops->u.has_strcpy)
                            {
                                pChannel->ops->event_strcpy(&bufferContext, pDataDesc[i].Data, pDataDesc[i].Size);
                            }
                            else
                            {
                                pChannel->ops->event_write(&bufferContext, pDataDesc[i].Data, pDataDesc[i].Size);
                            }
                            break;
                        }
                        case lttngh_DataType_StringUtf16Transcoded:
                        case lttngh_DataType_StringUtf32Transcoded:
                        {
                            assert(pDataDesc[i].Length <= cbTranscodeScratch - 1);

                            uint16_t cbUtf8Written;
                            if (pDataDesc[i].Type == lttngh_DataType_StringUtf16Transcoded)
                            {
                                cbUtf8Written = (uint16_t)Utf16ToUtf8(
                                    pDataDesc[i].Data, pDataDesc[i].Size / sizeof(char16_t),
                                    pbTranscodeScratch, pDataDesc[i].Length);
                            }
                            else
                            {
                                cbUtf8Written = (uint16_t)Utf32ToUtf8(
                                    pDataDesc[i].Data, pDataDesc[i].Size / sizeof(char32_t),
                                    pbTranscodeScratch, pDataDesc[i].Length);
                            }

                            pbTranscodeScratch[cbUtf8Written] = 0;
                            size_t iNul = strlen((char *)pbTranscodeScratch);
                            if (caa_unlikely(iNul != pDataDesc[i].Length))
                            {
                                // The data was changed on another thread or was truncated at a multi-byte char, so append #s.
                                assert(iNul <= cbUtf8Written);
                                assert(cbUtf8Written < pDataDesc[i].Length);
                                memset(pbTranscodeScratch + iNul,
                                       '#', pDataDesc[i].Length - iNul);
                                pbTranscodeScratch[pDataDesc[i].Length] = 0;
                            }

                            pChannel->ops->event_write(&bufferContext, pbTranscodeScratch, cbUtf8Written + 1u);
                            break;
                        }
                        case lttngh_DataType_SequenceUtf16Transcoded:
                        case lttngh_DataType_SequenceUtf32Transcoded:
                        {
                            assert(pDataDesc[i].Length <= cbTranscodeScratch - sizeof(uint16_t));

                            uint16_t cbUtf8Written;
                            if (pDataDesc[i].Type == lttngh_DataType_SequenceUtf16Transcoded)
                            {
                                cbUtf8Written = (uint16_t)Utf16ToUtf8(
                                    pDataDesc[i].Data, pDataDesc[i].Size / sizeof(char16_t),
                                    pbTranscodeScratch + sizeof(uint16_t), pDataDesc[i].Length);
                            }
                            else
                            {
                                cbUtf8Written = (uint16_t)Utf32ToUtf8(
                                    pDataDesc[i].Data, pDataDesc[i].Size / sizeof(char32_t),
                                    pbTranscodeScratch + sizeof(uint16_t), pDataDesc[i].Length);
                            }

                            if (caa_unlikely(cbUtf8Written != pDataDesc[i].Length))
                            {
                                // The data was changed on another thread or was truncated at a multi-byte char, so append #s.
                                assert(cbUtf8Written < pDataDesc[i].Length);
                                memset(pbTranscodeScratch + sizeof(uint16_t) + cbUtf8Written,
                                       '#', pDataDesc[i].Length - (unsigned)cbUtf8Written);
                                cbUtf8Written = pDataDesc[i].Length;
                            }

                            memcpy(pbTranscodeScratch, &cbUtf8Written, sizeof(uint16_t)); // Fill in length.
                            pChannel->ops->event_write(&bufferContext, pbTranscodeScratch, sizeof(uint16_t) + cbUtf8Written);
                            break;
                        }
                        default:
                        {
                            pChannel->ops->event_write(&bufferContext, pDataDesc[i].Data, pDataDesc[i].Size);
                            break;
                        }
                        }
                    }

                    pChannel->ops->event_commit(&bufferContext);
                }
            }
        } while ((++pTpProbe)->data);
    }

Done:

    tp_rcu_read_unlock_bp();

    if (caa_unlikely(pbTranscodeScratch != transcodeScratchOnStack))
    {
        free(pbTranscodeScratch);
    }

    return err;
}
