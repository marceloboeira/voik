# loglady
> loglady has a story for you

## Project Goals

* Learn and have fun™️
* Implement a Kinesis-like streaming-service
* Single binary
* Easy to Host, Run & Operate (have you tried to run Kafka yourself?)
  * Kubernetes friendly

## Roadmap

* [ ] Settle for a name
* [ ] Core CommitLog
  * [ ] Segments
    * [x] Write to segments
    * [x] Write across segments
    * [x] Read from segment
    * [ ] Read across segments
      * [ ] Index
    * [ ] Memory Mapped IO (Performance test before/after)
    * [ ] Non-volatire storage (read from disk) Ref: (https://github.com/zowens/commitlog/blob/master/src/file_set.rs#L17-L98)
  * [ ] Topics/Streams (probably should come up with a better name)
    * [ ] Partitions/Shards
      * [ ] ?
  * [ ] Performance tests
    * [ ] Write
    * [ ] Read
* [ ] Networking
  * [ ] Decide simple inicial protocol
  * [ ] Implement Basic Socket Communication
  * [ ] Implement TCP/HTTP?
  * [ ] Write to segments over the network
* [ ] Configuration
  * [ ] CLI Basics
* [ ] CI/Tooling
  * [ ] Setup Travis
    * [ ] Build with Linux/macOS
  * [ ] Docker
    * [ ] Alpine
* [ ] Testing
  * [ ] Benchmarks
  * [ ] Setup Integration Tests
  * [ ] Abstract Test Code
* [ ] Documentation
  * [ ] Document decisions
    * [ ] Document data structures / types
  * [ ] Improve README
    * [ ] How to Run

### Commands

* `make build` -> build
* `make build_release` -> build for release
* `make run` -> build and run
* `make install` -> use cargo to install locally
* `make test_watcher` - Run the tests under a watcher.
* `make docker_test_watcher` - Run the tests under a watcher on Docker (to ensure linux compatibility).

## References

* Blogposts
  * [How Kafka's Storage Internals Work](https://thehoard.blog/how-kafkas-storage-internals-work-3a29b02e026)
* Code
  * [travisjeffery/Jocko](https://github.com/travisjeffery/jocko) - Distributed commit log service in Go
  * [zowens/commitlog](http://github.com/zowens/commitlog) - Append-only commit log library for Rust
