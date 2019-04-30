[![Build Status](https://mscodehub.visualstudio.com/Azile/_apis/build/status/github-TraceLogging-CI?branchName=master)](https://mscodehub.visualstudio.com/Azile/_build/latest?definitionId=920&branchName=master)

# Dependencies

To use this library, you will need liblttng-ust-dev >= 2.10.

For Ubuntu 16.04 and below, you will need to add a PPA that includes this dependency:
sudo apt-add-repository ppa:lttng/stable-2.10 -y
sudo apt -y update

Then the final step is to install the dependent library:
sudo apt install liblttng-ust-dev

# Integration

Add this project as a subdirectory in your project, either as a git submodule or copying the code directly.
Then add that directory to your top-level CMakeLists.txt with 'add_subdirectory'.
This will make the target 'tracelogging' available.  
Add that to the target_link_libraries of any target that will use TraceLogging.

# Usage

At global scope, use TRACELOGGING_DEFINE_PROVIDER to definie a provider handle.
Then at runtime, before writing any events, call TraceLoggingRegister.
To write events, use TraceLoggingWrite.
Finally, at shutdown, use TraceLoggingUnregister.

# Contributing

This project welcomes contributions and suggestions.  Most contributions require you to agree to a
Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us
the rights to use your contribution. For details, visit https://cla.microsoft.com.

When you submit a pull request, a CLA-bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately (e.g., label, comment). Simply follow the instructions
provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or
contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.
