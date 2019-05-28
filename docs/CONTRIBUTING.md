# Contributing

Found a problem? Have a question? [open an issue](https://github.com/14-bits/voik/issues/new).

We also love pull requests from everyone, feel free to open one, but remember to follow the guidelines:

1. Double check the [roadmap](https://github.com/14-bits/voik/issues/7), there might be someone already working on similar tasks.

2. Fork, then clone the repo:

```
git clone git@github.com:your-username/voik.git
```

3. Set up your machine:

```
make build
````

4. Make sure the tests pass:

```
make test
```

5. Make your changes. Add tests for your changes. Make the tests pass:

```
make test
```

**ProTip**: you can use `make test_watcher` to run the tests on every change.

6. Push to your fork and [submit a pull request][pr].

[pr]: https://github.com/14-bits/voik/compare/

At this point you're waiting on us. We may suggest some changes or improvements or alternatives.

Some things that will increase the chance that your pull request is accepted:

* Write tests.
* Make sure performance is not degraded.
* Follow our [style guide][style].
* Write a [good commit message][commit].
* Rebase against master.

[style]: https://github.com/rust-lang/rustfmt
[commit]: http://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html
