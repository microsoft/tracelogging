get_filename_component(TRACELOGGING_CMAKE_DIR "${CMAKE_CURRENT_LIST_FILE}" PATH)
include(CMakeFindDependencyMacro)

# The exported lttngh library links LTTng::UST and, as a link-only dependency of
# the static library, lttng-ust-tracepoint::lttng-ust-tracepoint. Find both with
# the modules installed next to this file.
list(PREPEND CMAKE_MODULE_PATH "${TRACELOGGING_CMAKE_DIR}")
find_dependency(LTTngUST REQUIRED)
find_dependency(lttng-ust-tracepoint REQUIRED)
list(REMOVE_AT CMAKE_MODULE_PATH 0)

if (NOT TARGET tracelogging::tracelogging)
    include("${TRACELOGGING_CMAKE_DIR}/traceloggingTargets.cmake")
endif ()
