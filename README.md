# hvm-bench

Compare benchmarks of `hvm-core` versions.

Important hashes
- Modern: `09a3791cd8194fef28be95305835d4851eb0a854`
- Post ptr-refactor: `9bdbdcbe0816345545a3adf00704f9f4f01dcfe7`
- Pre ptr-refactor:  `c610b490fb071b7c9891b674bf399addaff3a580`
- Before dup-ref: `1a1e56327479a2681b1cbee2e0fc121b4c19cc14`
- Before compiler: `fb5a2a98d4ec19e4b4a4898a8124fa4a76f87ee1`

```
modern: cargo run -- run file.hvmc -s
```