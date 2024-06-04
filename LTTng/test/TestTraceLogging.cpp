// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#include <tracelogging/TraceLoggingProvider.h>

TRACELOGGING_DECLARE_PROVIDER(TestProviderCpp);
#define TestProvider TestProviderCpp
#include "TestTraceLogging.h"

TRACELOGGING_DEFINE_PROVIDER(
    TestProviderCpp,
    "TestProviderCpp",
    // {3f3dc547-92d7-59d6-ed26-053336a36f9b}
    (0x3f3dc547, 0x92d7, 0x59d6, 0xed, 0x26, 0x05, 0x33, 0x36, 0xa3, 0x6f, 0x9b));

static int TestTraceLoggingValue()
{
    void const* const pSamplePtr = (void*)(intptr_t)(-12345);

    TraceLoggingWrite(TestProvider, "Value:bool",
        TraceLoggingValue(false),
        TraceLoggingValue(true));
    TraceLoggingWrite(TestProvider, "Value:char",
        TraceLoggingValue('\0', "0"),
        TraceLoggingValue('A', "A"));
    TraceLoggingWrite(TestProvider, "Value:char16",
        TraceLoggingValue(u'\0', "0"),
        TraceLoggingValue(u'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:char32",
        TraceLoggingValue(U'\0', "0"),
        TraceLoggingValue(U'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:wchar",
        TraceLoggingValue(L'\0', "0"),
        TraceLoggingValue(L'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:schar",
        TraceLoggingValue((signed char)0, "0"),
        TraceLoggingValue((signed char)L'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:uchar",
        TraceLoggingValue((unsigned char)0, "0"),
        TraceLoggingValue((unsigned char)L'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:sshort",
        TraceLoggingValue((signed short)0, "0"),
        TraceLoggingValue((signed short)L'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:ushort",
        TraceLoggingValue((unsigned short)0, "0"),
        TraceLoggingValue((unsigned short)L'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:sint",
        TraceLoggingValue((signed int)0, "0"),
        TraceLoggingValue((signed int)L'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:uint",
        TraceLoggingValue((unsigned int)0, "0"),
        TraceLoggingValue((unsigned int)L'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:slong",
        TraceLoggingValue((signed long)0, "0"),
        TraceLoggingValue((signed long)L'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:ulong",
        TraceLoggingValue((unsigned long)0, "0"),
        TraceLoggingValue((unsigned long)L'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:slonglong",
        TraceLoggingValue((signed long long)0, "0"),
        TraceLoggingValue((signed long long)L'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:ulonglong",
        TraceLoggingValue((unsigned long long)0, "0"),
        TraceLoggingValue((unsigned long long)L'A', "A"));
    TraceLoggingWrite(TestProvider, "Value:float",
        TraceLoggingValue(0.0f, "0"),
        TraceLoggingValue(65.0f, "65"));
    TraceLoggingWrite(TestProvider, "Value:double",
        TraceLoggingValue(0.0, "0"),
        TraceLoggingValue(65.0, "65"));
    TraceLoggingWrite(TestProvider, "Value:void*",
        TraceLoggingValue((void*)0, "0"),
        TraceLoggingValue((void*)pSamplePtr, "p"));
    TraceLoggingWrite(TestProvider, "Value:cvoid*",
        TraceLoggingValue((void const*)0, "0"),
        TraceLoggingValue((void const*)pSamplePtr, "p"));
    TraceLoggingWrite(TestProvider, "Value:char*",
        TraceLoggingValue((char*)0, "0"),
        TraceLoggingValue((char*)"hello", "hello"));
    TraceLoggingWrite(TestProvider, "Value:cchar*",
        TraceLoggingValue((char const*)0, "0"),
        TraceLoggingValue((char const*)"hello", "hello"));
    TraceLoggingWrite(TestProvider, "Value:char16_t*",
        TraceLoggingValue((char16_t*)0, "0"),
        TraceLoggingValue((char16_t*)u"hello", "hello"));
    TraceLoggingWrite(TestProvider, "Value:cchar16_t*",
        TraceLoggingValue((char16_t const*)0, "0"),
        TraceLoggingValue((char16_t const*)u"hello", "hello"));
    TraceLoggingWrite(TestProvider, "Value:char32_t*",
        TraceLoggingValue((char32_t*)0, "0"),
        TraceLoggingValue((char32_t*)U"hello", "hello"));
    TraceLoggingWrite(TestProvider, "Value:cchar32_t*",
        TraceLoggingValue((char32_t const*)0, "0"),
        TraceLoggingValue((char32_t const*)U"hello", "hello"));
    TraceLoggingWrite(TestProvider, "Value:wchar_t*",
        TraceLoggingValue((wchar_t*)0, "0"),
        TraceLoggingValue((wchar_t*)L"hello", "hello"));
    TraceLoggingWrite(TestProvider, "Value:cwchar_t*",
        TraceLoggingValue((wchar_t const*)0, "0"),
        TraceLoggingValue((wchar_t const*)L"hello", "hello"));

    return 0;
}

#include <stdio.h>

// Returns 0 on success
int TestCpp()
{
    int err = TraceLoggingRegister(TestProvider);
    if (err != 0)
    {
        printf("Error: TestCpp register: %d\n", err);
        return err;
    }

    printf("Name: %s\n", TraceLoggingProviderName(TestProvider));

    err = TraceLoggingEventEnabled(TestProvider, "Event1");
    if (err == 0)
    {
        printf("Warning: Event1 is not enabled: %d\n", err);
    }
    TraceLoggingWrite(TestProvider, "Event1");

    err = TraceLoggingEventEnabled(TestProvider, "Event2");
    if (err == 0)
    {
        printf("Warning: Event2 is not enabled: %d\n", err);
    }
    TraceLoggingWrite(TestProvider, "Event2", TraceLoggingKeyword(3));

    err = TestCommon();
    int err_value = TestTraceLoggingValue();

    int err2 = TraceLoggingUnregister(TestProvider);
    if (err != 0)
    {
        puts("Error: TestCommon failed in the C++ tests");
        return err;
    }

    if (err_value != 0)
    {
        puts("Error: TestTraceLoggingValue failed");
        return err_value;
    }

    if (err2 != 0)
    {
        printf("Error: TestCpp unregister: %d\n", err2);
    }

    return err2;
}
