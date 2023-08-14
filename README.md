# Parsing and formatting for the HTTP Date: header

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

The date header is technically supposed to contain an IMF-fixdate value, but three formats
actually exist in the wild. This crate attempts parsing all three when calling parse.

This is a fork of <https://github.com/pyfisch/httpdate> to fix some things that I found mildily annoying while using it.

Changes include:

- This crate formats into an &mut \[u8\], for easier use with &\[std::io::IoSlice\].
- This crate parses from an &\[u8\] rather than a &str, since that is what you normally have when parsing http headers.
- This crate is no_std.
- This crate performs no allocations.
- This crate does not panic when given timestamps after the year 9999.
- The code for this crate is simpler. (Subjective, but I think most would agree.)
- This crate is more comprehensively tested with proptest and more edge tests.
	- The original crate used fuzz, which I've never used but it *seems* to only test truly random inputs. This means the fuzz tests never actually exercised the parsing code since none of the inputs were ever a valid value. (The chances of randomly generating 29 bytes that happen to be a valid timestamp are effectively impossible.)
	- The fuzz/prop tests in this crate generate random valid inputs and assert that the parsing succeeds and that parsing/decoding is invariant.
	- I also included more manually-chosen edge case tests around the epoch, the maximum representable date in IMF-fixdate format, and leap years.
- This crate validates correctness of the (redundant) weekday portion of the date header analytically rather than by converting the parsed value into a SystemTime and back into a second parsed value (with the correct weekday), then checking that the two weekdays match.
- Criterion reports improvement on 3 of 4 benchmarks of around -65%, though I'm not really sure why, it seems like too big of an improvement just for not doing the SystemTime conversion.
	- The fourth benchmark also improved by a similar amount, but it appears to not work correctly in the original crate so I don't include it.
	- I didn't fork for performance reasons, so I'm not too concerned about the precise improvements.

Here's a link to pyfisch's blog post on the original crate: <https://pyfisch.org/blog/http-datetime-handling/>
