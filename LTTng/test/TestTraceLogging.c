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

bool TestC()
{
    int err;
    err = TraceLoggingRegister(TestProvider);
    printf("TestC register: %d\n", err);
    printf("Name: %s\n", TraceLoggingProviderName(TestProvider));
    TraceLoggingWrite(TestProvider, "Event1");
    printf("Enabled1: %d\n", TraceLoggingEventEnabled(TestProvider, "Event1"));
    TraceLoggingWrite(TestProvider, "Event2", TraceLoggingKeyword(3));
    printf("Enabled2: %d\n", TraceLoggingEventEnabled(TestProvider, "Event2"));
    bool ok = TestCommon() && err == 0;
    err = TraceLoggingUnregister(TestProvider);
    printf("TestC unregister: %d\n", err);
    return ok && err == 0;
}
