#include <arpa/inet.h>

#ifdef __cplusplus
extern "C" {
#endif

    bool TestC(void);
    bool TestCpp(void);

#ifdef __cplusplus
} // extern "C"
#endif

typedef struct Buffer
{
    void* Buffer;
    uint16_t Length;
} Buffer;

static bool TestCommon(void)
{
    const bool b0 = 0;
    const bool b1 = 1;
    const uint8_t b8 = 1;
    const int32_t b32 = 1;
    const int8_t i8 = 100;
    const uint8_t u8 = 200;
    const int16_t i16 = 30000;
    const uint16_t u16 = 60000;
    const int32_t i32 = 2000000000;
    const uint32_t u32 = 4000000000;
    const long iL = 2000000000;
    const unsigned long uL = 4000000000;
    const int64_t i64 = 9000000000000000000;
    const uint64_t u64 = 18000000000000000000u;
    const float f32 = 3.14f;
    const double f64 = 6.28;
    const char ch = 'A';
    const char16_t u16ch = u'A';
    const char32_t u32ch = U'A';
    const wchar_t wch = L'B';
    const intptr_t iptr = 1234;
    const uintptr_t uptr = 4321;
    const uint32_t ft[] = { 10000, 20000 };
    const uint16_t st[] = { 45, 1, 2, 3, 4, 0, 0, 0 };
    char ch10[10] = "HowAreU8?";
    char16_t u16ch10[10] = u"HowAreU16";
    char32_t u32ch10[10] = U"HowAreU32";
    wchar_t wch10[10] = L"Goodbye!!";
    uint8_t const guid[16] = { 1,2,3,4,5,6,7,8,1,2,3,4,5,6,7,8};
    void const* const pSamplePtr = (void*)(intptr_t)(-12345);
    Buffer buf = { ch10, 4 };
    unsigned short n1 = 1;
    unsigned short n5 = 5;
    const uint16_t port80 = htons(80);

    const uint8_t ipv4data[] = { 127, 0, 0, 1 };
    in_addr_t ipv4;
    memcpy(&ipv4, ipv4data, sizeof(ipv4));

    const uint8_t ipv6data[] = { 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16 };
    struct in6_addr ipv6;
    memcpy(&ipv6, ipv6data, sizeof(ipv6));

    const struct sockaddr_in saIPv4 = {
        .sin_family = AF_INET,
        .sin_port = port80,
        .sin_addr = { ipv4 },
        .sin_zero = { 0 }
    };

    const struct sockaddr_in6 saIPv6 = {
        .sin6_family = AF_INET6,
        .sin6_port = port80,
        .sin6_flowinfo = 5,
        .sin6_addr = ipv6,
        .sin6_scope_id = htonl(1234)
    };

    TraceLoggingWrite(TestProvider, "CScalars1");
    TraceLoggingWrite(
        TestProvider,
        "CScalars2",
        TraceLoggingLevel(1),
        TraceLoggingKeyword(0x5),
        TraceLoggingOpcode(2));
    TraceLoggingWrite(
        TestProvider,
        "CScalars3",
        TraceLoggingLevel(2),
        TraceLoggingKeyword(0x80),
        TraceLoggingOpcode(3),
        TraceLoggingLevel(4),
        TraceLoggingKeyword(0x05),
        TraceLoggingChannel(11),
        TraceLoggingEventTag(0x1234),
        TraceLoggingDescription("Hello"),
        TraceLoggingCustomAttribute("custom", "attribute"),
        TraceLoggingStruct(1, "IgnoredStruct"),
            TraceLoggingInt32(1));

    uint8_t oldActivityId[16];
    lttngh_ActivityIdGet(oldActivityId);
    uint8_t activityId[16];
    lttngh_ActivityIdCreate(activityId);
    TraceLoggingWrite(TestProvider, "ThreadActivity0");
    lttngh_ActivityIdSet(guid);
    TraceLoggingWrite(TestProvider, "ThreadActivity1");
    lttngh_ActivityIdSet(oldActivityId);
    TraceLoggingWrite(TestProvider, "ThreadActivity2");
    TraceLoggingWriteActivity(TestProvider, "Transfer00", NULL, NULL);
    TraceLoggingWriteActivity(TestProvider, "Transfer01", NULL, guid);
    TraceLoggingWriteActivity(TestProvider, "Transfer10", guid, NULL);
    TraceLoggingWriteActivity(TestProvider, "Transfer11", guid, guid);
    TraceLoggingWriteActivity(TestProvider, "TransferOO", oldActivityId, oldActivityId);

    TraceLoggingWrite(TestProvider, "i8", TraceLoggingInt8(i8));
    TraceLoggingWrite(TestProvider, "u8", TraceLoggingUInt8(u8));
    TraceLoggingWrite(TestProvider, "i16", TraceLoggingInt16(i16));
    TraceLoggingWrite(TestProvider, "u16", TraceLoggingUInt16(u16));
    TraceLoggingWrite(TestProvider, "i32", TraceLoggingInt32(i32));
    TraceLoggingWrite(TestProvider, "u32", TraceLoggingUInt32(u32));
    TraceLoggingWrite(TestProvider, "iL", TraceLoggingLong(iL));
    TraceLoggingWrite(TestProvider, "uL", TraceLoggingULong(uL));
    TraceLoggingWrite(TestProvider, "i64", TraceLoggingInt64(i64));
    TraceLoggingWrite(TestProvider, "u64", TraceLoggingUInt64(u64));

    TraceLoggingWrite(TestProvider, "hi8",  TraceLoggingHexInt8(i8));
    TraceLoggingWrite(TestProvider, "hu8",  TraceLoggingHexUInt8(u8));
    TraceLoggingWrite(TestProvider, "hi16", TraceLoggingHexInt16(i16));
    TraceLoggingWrite(TestProvider, "hu16", TraceLoggingHexUInt16(u16));
    TraceLoggingWrite(TestProvider, "hi32", TraceLoggingHexInt32(i32));
    TraceLoggingWrite(TestProvider, "hu32", TraceLoggingHexUInt32(u32));
    TraceLoggingWrite(TestProvider, "hiL",  TraceLoggingHexLong(iL));
    TraceLoggingWrite(TestProvider, "huL",  TraceLoggingHexULong(uL));
    TraceLoggingWrite(TestProvider, "hi64", TraceLoggingHexInt64(i64));
    TraceLoggingWrite(TestProvider, "hu64", TraceLoggingHexUInt64(u64));

    TraceLoggingWrite(TestProvider, "iptr", TraceLoggingIntPtr(iptr));
    TraceLoggingWrite(TestProvider, "uptr", TraceLoggingUIntPtr(uptr));
    TraceLoggingWrite(TestProvider, "f32", TraceLoggingFloat32(f32));
    TraceLoggingWrite(TestProvider, "f64", TraceLoggingFloat64(f64));
    TraceLoggingWrite(TestProvider, "b8", TraceLoggingBoolean(b0), TraceLoggingBoolean(b1));
    TraceLoggingWrite(TestProvider, "b32", TraceLoggingBool(b0), TraceLoggingBool(b1));

    TraceLoggingWrite(TestProvider, "ch", TraceLoggingChar(ch));
    TraceLoggingWrite(TestProvider, "wch", TraceLoggingWChar(wch));
    TraceLoggingWrite(TestProvider, "u16ch", TraceLoggingChar16(u16ch));
    TraceLoggingWrite(TestProvider, "u32ch", TraceLoggingChar16(u32ch));
    TraceLoggingWrite(TestProvider, "ptr", TraceLoggingPointer(pSamplePtr));
    TraceLoggingWrite(TestProvider, "cptr", TraceLoggingCodePointer(pSamplePtr));
    TraceLoggingWrite(TestProvider, "pid", TraceLoggingPid(u32));
    TraceLoggingWrite(TestProvider, "tid", TraceLoggingTid(u32));
    TraceLoggingWrite(TestProvider, "port", TraceLoggingPort(port80));
    TraceLoggingWrite(TestProvider, "ipV4", TraceLoggingIPv4(ipv4), TraceLoggingChar(ch));
    TraceLoggingWrite(TestProvider, "ipV6", TraceLoggingIPv6(&ipv6, "ipv6"), TraceLoggingChar(ch));
    TraceLoggingWrite(TestProvider, "saV4", TraceLoggingSocketAddress(&saIPv4, sizeof(saIPv4), "saV4"), TraceLoggingChar(ch));
    TraceLoggingWrite(TestProvider, "saV6", TraceLoggingSocketAddress(&saIPv6, sizeof(saIPv6), "saV6"), TraceLoggingChar(ch));
    TraceLoggingWrite(TestProvider, "saEmpty", TraceLoggingSocketAddress("", 0, "empty"), TraceLoggingChar(ch));
    TraceLoggingWrite(TestProvider, "saGarbage", TraceLoggingSocketAddress("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789", 36, "garbage"), TraceLoggingChar(ch));
    TraceLoggingWrite(TestProvider, "winerror", TraceLoggingWinError(u32));
    TraceLoggingWrite(TestProvider, "ntstatus", TraceLoggingNTStatus(u32));
    TraceLoggingWrite(TestProvider, "hresult", TraceLoggingHResult(u32));

    TraceLoggingWrite(TestProvider, "guid", TraceLoggingGuid(guid));
    TraceLoggingWrite(TestProvider, "st", TraceLoggingSystemTime(st));
    TraceLoggingWrite(TestProvider, "ust", TraceLoggingSystemTimeUtc(st));
    TraceLoggingWrite(TestProvider, "ft", TraceLoggingFileTime(ft));
    TraceLoggingWrite(TestProvider, "uft", TraceLoggingFileTimeUtc(ft));

    TraceLoggingWrite(TestProvider, "sz",
        TraceLoggingString(NULL, "NULL"),
        TraceLoggingString(ch10));
    TraceLoggingWrite(TestProvider, "sz8",
        TraceLoggingUtf8String(NULL, "NULL"),
        TraceLoggingUtf8String(ch10));
    TraceLoggingWrite(TestProvider, "wsz",
        TraceLoggingWideString(NULL, "NULL"),
        TraceLoggingWideString(wch10));
    TraceLoggingWrite(TestProvider, "sz16",
        TraceLoggingString16(NULL, "NULL"),
        TraceLoggingString16(u16ch10));
    TraceLoggingWrite(TestProvider, "sz32",
        TraceLoggingString32(NULL, "NULL"),
        TraceLoggingString32(u32ch10));

    TraceLoggingWrite(TestProvider, "csz",
        TraceLoggingCountedString(NULL, 0, "NULL"),
        TraceLoggingCountedString(ch10, 5));
    TraceLoggingWrite(TestProvider, "csz8",
        TraceLoggingCountedUtf8String(NULL, 0, "NULL"),
        TraceLoggingCountedUtf8String(ch10, 5));
    TraceLoggingWrite(TestProvider, "cwsz",
        TraceLoggingCountedWideString(NULL, 0, "NULL"),
        TraceLoggingCountedWideString(wch10, 5));
    TraceLoggingWrite(TestProvider, "csz16",
        TraceLoggingCountedString16(NULL, 0, "NULL"),
        TraceLoggingCountedString16(u16ch10, 5));
    TraceLoggingWrite(TestProvider, "csz32",
        TraceLoggingCountedString32(NULL, 0, "NULL"),
        TraceLoggingCountedString32(u32ch10, 5));

    TraceLoggingWrite(TestProvider, "bin",
        TraceLoggingBinary(NULL, 0, "NULL"),
        TraceLoggingBinary(ch10, 5),
        TraceLoggingBinaryBuffer(&buf, Buffer, "buf"));

    TraceLoggingWrite(TestProvider, "ai8",
        TraceLoggingInt8FixedArray(&i8, 1, "a1"),
        TraceLoggingInt8Array(&i8, n1, "s"));
    TraceLoggingWrite(TestProvider, "au8",
        TraceLoggingUInt8FixedArray(&u8, 1, "a1"),
        TraceLoggingUInt8Array(&u8, n1, "s"));
    TraceLoggingWrite(TestProvider, "ai16",
        TraceLoggingInt16FixedArray(&i16, 1, "a1"),
        TraceLoggingInt16Array(&i16, n1, "s"));
    TraceLoggingWrite(TestProvider, "au16",
        TraceLoggingUInt16FixedArray(&u16, 1, "a1"),
        TraceLoggingUInt16Array(&u16, n1, "s"));
    TraceLoggingWrite(TestProvider, "ai32",
        TraceLoggingInt32FixedArray(&i32, 1, "a1"),
        TraceLoggingInt32Array(&i32, n1, "s"));
    TraceLoggingWrite(TestProvider, "au32",
        TraceLoggingUInt32FixedArray(&u32, 1, "a1"),
        TraceLoggingUInt32Array(&u32, n1, "s"));
    TraceLoggingWrite(TestProvider, "aiL",
        TraceLoggingLongFixedArray(&iL, 1, "a1"),
        TraceLoggingLongArray(&iL, n1, "s"));
    TraceLoggingWrite(TestProvider, "auL",
        TraceLoggingULongFixedArray(&uL, 1, "a1"),
        TraceLoggingULongArray(&uL, n1, "s"));
    TraceLoggingWrite(TestProvider, "ai64",
        TraceLoggingInt64FixedArray(&i64, 1, "a1"),
        TraceLoggingInt64Array(&i64, n1, "s"));
    TraceLoggingWrite(TestProvider, "au64",
        TraceLoggingUInt64FixedArray(&u64, 1, "a1"),
        TraceLoggingUInt64Array(&u64, n1, "s"));

    TraceLoggingWrite(TestProvider, "hai8",
        TraceLoggingHexInt8FixedArray(&i8, 1, "a1"),
        TraceLoggingHexInt8Array(&i8, n1, "s"));
    TraceLoggingWrite(TestProvider, "hau8",
        TraceLoggingHexUInt8FixedArray(&u8, 1, "a1"),
        TraceLoggingHexUInt8Array(&u8, n1, "s"));
    TraceLoggingWrite(TestProvider, "hai16",
        TraceLoggingHexInt16FixedArray(&i16, 1, "a1"),
        TraceLoggingHexInt16Array(&i16, n1, "s"));
    TraceLoggingWrite(TestProvider, "hau16",
        TraceLoggingHexUInt16FixedArray(&u16, 1, "a1"),
        TraceLoggingHexUInt16Array(&u16, n1, "s"));
    TraceLoggingWrite(TestProvider, "hai32",
        TraceLoggingHexInt32FixedArray(&i32, 1, "a1"),
        TraceLoggingHexInt32Array(&i32, n1, "s"));
    TraceLoggingWrite(TestProvider, "hau32",
        TraceLoggingHexUInt32FixedArray(&u32, 1, "a1"),
        TraceLoggingHexUInt32Array(&u32, n1, "s"));
    TraceLoggingWrite(TestProvider, "haiL",
        TraceLoggingHexLongFixedArray(&iL, 1, "a1"),
        TraceLoggingHexLongArray(&iL, n1, "s"));
    TraceLoggingWrite(TestProvider, "hauL",
        TraceLoggingHexULongFixedArray(&uL, 1, "a1"),
        TraceLoggingHexULongArray(&uL, n1, "s"));
    TraceLoggingWrite(TestProvider, "hai64",
        TraceLoggingHexInt64FixedArray(&i64, 1, "a1"),
        TraceLoggingHexInt64Array(&i64, n1, "s"));
    TraceLoggingWrite(TestProvider, "hau64",
        TraceLoggingHexUInt64FixedArray(&u64, 1, "a1"),
        TraceLoggingHexUInt64Array(&u64, n1, "s"));

    TraceLoggingWrite(TestProvider, "aiptr",
        TraceLoggingIntPtrFixedArray(&iptr, 1, "a1"),
        TraceLoggingIntPtrArray(&iptr, n1, "s"));
    TraceLoggingWrite(TestProvider, "auptr",
        TraceLoggingUIntPtrFixedArray(&uptr, 1, "a1"),
        TraceLoggingUIntPtrArray(&uptr, n1, "s"));
    TraceLoggingWrite(TestProvider, "ab32",
        TraceLoggingBoolFixedArray(&b32, 1, "a1"),
        TraceLoggingBoolArray(&b32, n1, "s"));
    TraceLoggingWrite(TestProvider, "ab8",
        TraceLoggingBooleanFixedArray(&b8, 1, "a1"),
        TraceLoggingBooleanArray(&b8, n1, "s"));

    TraceLoggingWrite(TestProvider, "ach",
        TraceLoggingCharFixedArray(ch10, 4, "a4"),
        TraceLoggingCharArray(ch10, n5, "s5"));
    TraceLoggingWrite(TestProvider, "awch",
        TraceLoggingWCharFixedArray(wch10, 4, "a4"),
        TraceLoggingWCharArray(wch10, n5, "s5"));
    TraceLoggingWrite(TestProvider, "ach16",
        TraceLoggingChar16FixedArray(u16ch10, 4, "a4"),
        TraceLoggingChar16Array(u16ch10, n5, "s5"));
    TraceLoggingWrite(TestProvider, "ach32",
        TraceLoggingChar32FixedArray(u32ch10, 4, "a4"),
        TraceLoggingChar32Array(u32ch10, n5, "s5"));

    TraceLoggingWrite(TestProvider, "aptr",
        TraceLoggingPointerFixedArray(&pSamplePtr, 1, "a1"),
        TraceLoggingPointerArray(&pSamplePtr, n1, "s"));
    TraceLoggingWrite(TestProvider, "acptr",
        TraceLoggingCodePointerFixedArray(&pSamplePtr, 1, "a1"),
        TraceLoggingCodePointerArray(&pSamplePtr, n1, "s"));

    TraceLoggingWrite(TestProvider, "aguid",
        TraceLoggingGuidFixedArray(guid, 1, "a1"),
        TraceLoggingGuidArray(guid, n1, "s"));
    TraceLoggingWrite(TestProvider, "ast",
        TraceLoggingSystemTimeFixedArray(st, 1, "a1"),
        TraceLoggingSystemTimeArray(st, n1, "s"));
    TraceLoggingWrite(TestProvider, "aust",
        TraceLoggingSystemTimeUtcFixedArray(st, 1, "a1"),
        TraceLoggingSystemTimeUtcArray(st, n1, "s"));
    TraceLoggingWrite(TestProvider, "aft",
        TraceLoggingFileTimeFixedArray(ft, 1, "a1"),
        TraceLoggingFileTimeArray(ft, n1, "s"));
    TraceLoggingWrite(TestProvider, "auft",
        TraceLoggingFileTimeUtcFixedArray(ft, 1, "a1"),
        TraceLoggingFileTimeUtcArray(ft, n1, "s"));

    return true;
}
