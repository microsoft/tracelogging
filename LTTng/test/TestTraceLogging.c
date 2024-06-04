// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#include <tracelogging/TraceLoggingProvider.h>

TRACELOGGING_DECLARE_PROVIDER(TestProviderC);
#define TestProvider TestProviderC
#include "TestTraceLogging.h"

TRACELOGGING_DEFINE_PROVIDER(
    TestProviderC,
    "TestProviderC",
    // {0da7a945-e9b1-510f-0ccf-ab1af0bc095b}
    (0x0da7a945, 0xe9b1, 0x510f, 0x0c, 0xcf, 0xab, 0x1a, 0xf0, 0xbc, 0x09, 0x5b));

#include <stdio.h>

// Returns 0 on success
int TestC()
{
    int err = TraceLoggingRegister(TestProvider);
    if (err != 0)
    {
        printf("Error: TestC register: %d\n", err);
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
    int err2 = TraceLoggingUnregister(TestProvider);
    if (err != 0)
    {
        puts("Error: TestCommon failed in the C tests");
        return err;
    }

    if (err2 != 0)
    {
        printf("Error: TestC unregister: %d\n", err2);
    }

    return err2;
}
