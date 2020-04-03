# Advent of Code 2019 - Rust
My Rust solutions to all [Advent of Code 2019](https://adventofcode.com/2019) problems. I'm new to Rust, but I've learned a whole lot in this process, and I'd like to think there's _some_ value to someone out there in seeing how I've done it.

My style prioritises being easy to follow, understand and reason about.  So you'll typically see a lot _more_ code here than with some other people's solutions - this isn't the place to come to learn how to write really tight, concise Rust that does a lot in few lines.  With the occasional exception, I generally haven't sacrificed much in the way of performance, though. Day 6 is a good example of this.
## Current status
Everything's as good as I'm planning to make it. Most days are worth looking at, but I'd skip days 18 and 20 where I haven't learned enough graph theory to write a performant solution, and day 25 is just I/O between the Intcode computer and human user, no automated gameplay. Aside from those three days, everything runs in under half a second _total_ - go Rust!

The Intcode computer now features three modes of operation: concurrent, synchronous, and async. Most Intcode days use concurrent, while day 23 demonstrates synchronous (with and without an async wrapper), and day 5 has a really noddy use of async.

Running times (best quartile) on my machine:

| Day | Time (ms) |
| --- | --------- |
| 1   | 0.1       |
| 2   | 9         |
| 3   | 0.3       |
| 4   | 48        |
| 5   | 0.1       |
| 6   | 13        |
| 7   | 72        |
| 8   | 0.25      |
| 9   | 5         |
| 10  | 10        |
| 11  | 12        |
| 12  | 9         |
| 13  | 23        |
| 14  | 0.5       |
| 15  | 6         |
| 16  | 84        |
| 17  | 3         |
| 18  | 165978 ( ͡° ͜ʖ ͡°) |
| 19  | 31        |
| 20  | ~1700     |
| 21  | 11        |
| 22  | 2         |
| 23  | 5         |
| 24  | 4         |
| 25  | N/A       |
