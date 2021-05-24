## Log-structured key-value store

In the style of [bitcask](https://github.com/basho/bitcask/blob/develop-3.0/doc/bitcask-intro.pdf).

(This loosely follows this [course](https://github.com/pingcap/talent-plan/tree/master/courses/rust/projects/project-2).)

This is now networked in a client-server model, and made asynchronous using
locks. Perhaps in the future I will switch to using `async`/`await` using
`tokio`.
