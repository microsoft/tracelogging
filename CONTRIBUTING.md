# Contributing to TraceLogging

## Reporting Issues

If you find a bug, please file an issue in GitHub via the [issues](https://github.com/Microsoft/TraceLogging/issues) page.

## Contributing

This project welcomes contributions and suggestions. Most contributions require you to
agree to a Contributor License Agreement (CLA) declaring that you have the right to,
and actually do, grant us the rights to use your contribution. For details, visit
[https://cla.microsoft.com/](https://cla.microsoft.com/).

When you submit a pull request, a CLA-bot will automatically determine whether you need
to provide a CLA and decorate the PR appropriately (e.g., label, comment). Simply follow the
instructions provided by the bot. You will only need to do this once across all repositories using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/)
or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

## Release

This repository uses the [GitHub Flow](https://guides.github.com/introduction/flow/) model.

This means that any commit which reaches master is considered 'released' since projects which depend on this library as a submodule may choose to update at any time. The master branch is policy-protected and does not accept direct pushes.

To get code into master:

1. Create a topic branch from master.
2. Commit code to your topic.
3. Open a Pull Request into master.
4. Once approved, squash your branch into master.

Pull requests must use squash commit to keep master history clean.

Pull Requests must be approved by a member of the [Device Health Services Team](https://github.com/orgs/microsoft/teams/device-health-services-team).

Pull Requests must be up to date with master and must pass the CI status check.
