# Datetime utils for HTTP.

[![Build Status](https://travis-ci.org/jayshua/date_header.svg?branch=master)](https://travis-ci.org/jayshua/date_header)
[![Crates.io](https://img.shields.io/crates/v/date_header.svg)](https://crates.io/crates/date_header)
[![Documentation](https://docs.rs/date_header/badge.svg)](https://docs.rs/date_header)

Multiple HTTP header fields store timestamps.
For example, a response created on May 15, 2015 may contain the header
`Date: Fri, 15 May 2015 15:34:21 GMT`. Since the timestamp does not
contain any timezone or leap second information, it is equvivalent to
writing 1431696861 Unix time.

This crate provides two functions:

* `parse` to parse an HTTP datetime string to a u64 unix timestamp
* `format` to format a u64 unix timestamp to an IMF-fixdate

This is a fork of <https://github.com/pyfisch/httpdate> to fix some things that I found mildily annoying while using it.

Changes include:

- This crate formats into an &mut [u8], for easier use with &[IoSlice].
- This crate parses from an &[u8] rather than a &str, since that is what you normally have when parsing http headers.
- This crate is no_std.
- This crate performs no allocations.
- The code for this crate is simpler. (Subjective, but I think most would agree.)
- This crate is more comprehensively tested with proptest and more edge tests.
	- The old crate used fuzz, which I've never used but I assume the purpose of is to only test
	truly random inputs. This means the fuzz tests never actually exercised the parsing code
	since none of the inputs were ever a valid value. (The chances of randomly generating 29 bytes
	that happen to be a valid timestamp are effectively impossible.)
	- The fuzz/prop tests in this crate generate random *valid* inputs and assert that the parsing
	succeeds and that parsing/decoding is invariant.
	- I also included more manually-chosen edge case tests around the epoch, the maximum
	representable date in IMF-fixdate format, and leap years.
- This crate has somewhere around 100 fewer lines of implementation code.
- These improvements(?) didn't come for free though. Unlike the
  original crate, this crate does *not* validate the redundant weekday field
  when parsing Date headers, it just ignores it. (Because it's redundant. The date is fully specified by the month-day-year portion of the header value.)
  The original crate validated the weekday field by formatting the parsed datetime (to get the correct weekday)
  then *reparsing* that formatted, value and checking that the second parse (of the correct weekday) matches the
  original parse. What you wanted was a parse. What you got was *two* parses, a string formatting, and an allocation.
- Criterion reports improvement on 3 of 4 benchmarks of around -65%, presumably due to performing 50% less parsing
  per parse and removing the allocation and format on each parse.
  The fourth benchmark appears to be broken in the original crate, so I'm not sure it makes sense to consider that one
  an improvement as well.

Here's a link to pyfisch's blog post on the original crate: <https://pyfisch.org/blog/http-datetime-handling/>
