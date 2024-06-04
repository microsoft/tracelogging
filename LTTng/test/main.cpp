extern "C" int TestC();
int TestCpp();

#include <stdio.h>

int main(int, char*[])
{
    int result = 0;
    int c_tests = TestC();
    if (c_tests != 0)
    {
        puts("C tests failed");
        result = c_tests;
    }

    int cpp_tests = TestCpp();
    if (cpp_tests != 0)
    {
        puts("C++ tests failed");
        result = cpp_tests;
    }

    return result;
}
