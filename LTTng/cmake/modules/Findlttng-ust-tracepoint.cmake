# Output target
#   lttng-ust-tracepoint::lttng-ust-tracepoint

include(AliasPkgConfigTarget)

if (TARGET lttng-ust-tracepoint::lttng-ust-tracepoint)
    return()
endif()

# First try and find with PkgConfig
find_package(PkgConfig QUIET)
if (PKG_CONFIG_FOUND)
    pkg_check_modules(lttng-ust-tracepoint QUIET IMPORTED_TARGET lttng-ust-tracepoint)
    if (TARGET PkgConfig::lttng-ust-tracepoint)
        alias_pkg_config_target(lttng-ust-tracepoint::lttng-ust-tracepoint PkgConfig::lttng-ust-tracepoint)
        return()
    endif ()
endif ()

# If that doesn't work, try again with old fashioned path lookup, with some caching
if (NOT lttng-ust-tracepoint_LIBRARY)
    find_library(lttng-ust-tracepoint_LIBRARY
        NAMES lttng-ust-tracepoint)

    include(FindPackageHandleStandardArgs)
    find_package_handle_standard_args(lttng-ust-tracepoint DEFAULT_MSG
        lttng-ust-tracepoint_LIBRARY)

    mark_as_advanced(lttng-ust-tracepoint_LIBRARY)
endif()

add_library(lttng-ust-tracepoint::lttng-ust-tracepoint UNKNOWN IMPORTED)
set_target_properties(lttng-ust-tracepoint::lttng-ust-tracepoint PROPERTIES
    IMPORTED_LOCATION "${lttng-ust-tracepoint_LIBRARY}")

set(lttng-ust-tracepoint_FOUND TRUE)
