# hvm-compare-perf

Compare benchmarks of `hvm-core` versions.

### How it works.

The directory `.bench-dir` includes a modified clone of the `hvm-core` repo. The `hvm-compare-perf` programs checks out different commits, which must be listed in `commits.cfg`, and times all programs in each commit.

Important hashes
- Modern: `09a3791cd8194fef28be95305835d4851eb0a854`
- Post ptr-refactor: `9bdbdcbe0816345545a3adf00704f9f4f01dcfe7`
- Pre ptr-refactor:  `c610b490fb071b7c9891b674bf399addaff3a580`
- Before dup-ref: `1a1e56327479a2681b1cbee2e0fc121b4c19cc14`
- Before compiler: `fb5a2a98d4ec19e4b4a4898a8124fa4a76f87ee1`

Unfortunately, the different versions of hvm-core are incompatible. For example, they might use different amounts of RAM on startup, or they might use different syntaxes for binary AND and OR. To patch this, I made temporary detached commits in the clone of the repo which make the different versions more compatible. This is why the hashes in `commits.cfg` are not actually the "important hashes", but rather childless commits which have those commits as their parent.


To do that, simply `git checkout` in `.bench-dir` to the commit you're testing, do your modifications, then `git add` and `git commit`. Copy the new hash and edit `commits.cfg` to put it in place of the original commit.