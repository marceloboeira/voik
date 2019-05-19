<p align="center">
  <img src="https://github.com/marceloboeira/voik/blob/master/docs/logos/github.png?raw=true" width="400">
  <p align="center">An experiment on persistent data-streaming<p>
  <p align="center">
    <img src="https://travis-ci.org/14-bits/voik.svg?branch=master">
  </p>
</p>

## Status

Currently, working in the foundation of the storage layer.

Checkout the Roadmap for feature-specific details.

## Project Goals

* Learn and have fun™️
* Implement a Kinesis-like streaming-service
* Single binary
* Easy to Host, Run & Operate (have you tried to run Kafka yourself?)
  * Kubernetes friendly

## Performance

These are preliminar and poorly collected results, yet it looks interesting:

**Storage** (Tests are completely offline, no network¹ ...)

```
Segment size: 20MiB
Index size: 10MiB
~15GiB worth records written in 157.005796s
~15GiB worth cold records read in 3.744316s
~15GiB worth warm records read in 2.448553s
```

Per-segment²:
* ~90 MiB/s on write
* ~390 MiB/s on cold read (while loading into memory pages)
* ~610 MiB/s on warm read (already loaded into memory pages)


Notes:
* ¹ - Offline - no network overhead taken into account, network will be a big player on the overhead. However, the focus now is storage.
* ² - Per-segment performance, in a comparinson with kinesis/kafka that would be the per-shard value. If you were to have 10 shards, you could achieve 10x that, limited by external factors, HD/CPU/...

Setup:

```
OS: macOS Mojave 10.14.4 (18E226)
CPU: 2,5 GHz Intel Core i7
RAM: 16 GB 2133 MHz LPDDR3
HD: 256 GB SSD Storage
```

## Roadmap

* [x] Settle for a name
* [ ] Storage
  * [ ] Core CommitLog
    * [ ] Segments
      * [x] Write to segments
      * [x] Write across segments
      * [x] Read from segment
      * [x] Read across segments
        * [x] Index
        * [ ] Better API for reading
      * [x] Extract logfile from segment
      * [x] Memory Mapped IO (Performance test before/after)
        * [x] Write
          * [x] Index
          * [x] Segment
          * [x] Refactor segment split to take index into account
        * [x] Read
          * [x] Index
          * [x] Segment
      * [ ] Restore state from disk - Ref: (https://github.com/zowens/commitlog/blob/master/src/file_set.rs#L17-L98)
    * [ ] Record Struct
      * [ ] End-to-end use of records
  * [ ] Topics/Streams (probably should come up with a better name)
    * [ ] Partitions/Shards
      * [ ] ?
* [ ] Networking/API
  * [ ] Decide simple inicial protocol
  * [ ] Implement Basic Socket Communication
  * [ ] Implement TCP/HTTP?
  * [ ] Write to streams over the network
  * [ ] zero-cost copy (OS sendfile)
* [ ] Replication
  * [ ] More to come here...
* [ ] Operational
  * [ ] Configuration
    * [ ] CLI Basics
* [ ] CI/Tooling
  * [ ] Setup Travis
    * [ ] Build with Linux/macOS
  * [ ] Docker
    * [ ] Alpine
* [ ] Testing
  * [ ] Benchmarks
    * [ ] Write
    * [ ] Read
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

* Talks
  *  [Building a Distributed Message Log from Scratch by Tyler Treat - Video](https://www.youtube.com/watch?v=oKbm9XFxB2k)
* Blogposts
  * [Building a Distributed Log from Scratch, Part 1: Storage Mechanics](https://bravenewgeek.com/building-a-distributed-log-from-scratch-part-1-storage-mechanics/)
  * [Building a Distributed Log from Scratch, Part 2: Data Replication](https://bravenewgeek.com/building-a-distributed-log-from-scratch-part-2-data-replication)
  * [Building a Distributed Log from Scratch, Part 3: Scaling Message Delivery](https://bravenewgeek.com/building-a-distributed-log-from-scratch-part-3-scaling-message-delivery/)
  * [Building a Distributed Log from Scratch, Part 4: Trade-Offs and Lessons Learned](https://bravenewgeek.com/building-a-distributed-log-from-scratch-part-4-trade-offs-and-lessons-learned/)
  * [Building a Distributed Log from Scratch, Part 5: Sketching a New System](https://bravenewgeek.com/building-a-distributed-log-from-scratch-part-5-sketching-a-new-system/)
  * [The Log: What every software engineer should know about real-time data's unifying abstraction](https://engineering.linkedin.com/distributed-systems/log-what-every-software-engineer-should-know-about-real-time-datas-unifying)
  * [How Kafka's Storage Internals Work](https://thehoard.blog/how-kafkas-storage-internals-work-3a29b02e026)
* Code
  * [travisjeffery/Jocko](https://github.com/travisjeffery/jocko) - Distributed commit log service in Go
  * [zowens/commitlog](http://github.com/zowens/commitlog) - Append-only commit log library for Rust
  * [liftbridge-io/liftbridge](http://github.com/liftbridge-io/liftbridge) - Lightweight, fault-tolerant message streams
