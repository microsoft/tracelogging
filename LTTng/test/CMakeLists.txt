cmake_minimum_required(VERSION 3.20)

add_executable(
    LTTngTest
    main.cpp
    TestTraceLogging.cpp
    TestTraceLogging.c)

target_link_libraries(
    LTTngTest
    PRIVATE
    tracelogging
    # lttng-ust-common # Needed for LTTNG 2.13+
    )

add_test(NAME LTTngTest COMMAND LTTngTest)

set (failRegex "Error" "Fail")
set_property (TEST LTTngTest
              PROPERTY FAIL_REGULAR_EXPRESSION "${failRegex}")
