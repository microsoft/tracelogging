vcpkg_from_github(
    OUT_SOURCE_PATH SOURCE_PATH
    REPO microsoft/tracelogging
    REF v0.3.1
    SHA512 8a34eb7d91b92c36fc0682d45c188361aaffd57b0b8bd62d16b876d3231fc96c5fe3369dc26d23ad2ca4eacda9ac565d04866c4b8b3be1214906c5f939d8a23a
    HEAD_REF master
)

vcpkg_configure_cmake(
	SOURCE_PATH "${SOURCE_PATH}"
	PREFER_NINJA 

    OPTIONS
        -DTRACELOGGING_BUILD_TESTS=OFF
        ${OPTIONS}
)

vcpkg_build_cmake()

vcpkg_fixup_cmake_targets()


file(REMOVE_RECURSE "${CURRENT_PACKAGES_DIR}/debug/include")

# ERROR: ${CURRENT_PACKAGES_DIR} = /opt/vcpkg/packages/tracelogging_x64-linux/debug/
# when it should be /opt/vcpkg/packages/tracelogging_x64-linux/
file(
	INSTALL "${SOURCE_PATH}/LICENSE"
	DESTINATION "${CURRENT_PACKAGES_DIR}/share/${PORT}"
	RENAME copyright)

vcpkg_install_cmake()