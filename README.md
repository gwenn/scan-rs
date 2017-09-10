Streaming/Lazy/No-copy scanner.

And a CSV reader implementation.

I tried [FallibleStreamingIterator](https://docs.rs/fallible-streaming-iterator/0.1.5/fallible_streaming_iterator/trait.FallibleStreamingIterator.html) but failed due to some errors reported by the borrow checker.
But our `Scanner` is a `FallibleStreamingIterator`:
> `FallibleStreamingIterator` differs from the standard library's `Iterator` trait in two ways: iteration can fail, resulting in an error, and only one element of the iteration is available at any time.
> While these iterators cannot be used with Rust `for` loops, `while let` loops offer a similar level of ergonomics.

Currently, there are one `unsafe` block in the `scan` method used to bypass the borrow checker.
I don't know if it can be replaced with safe code.
But I am quite confident that it is safe.