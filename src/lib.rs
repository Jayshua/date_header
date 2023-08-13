#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]


// Unix timestamp for Jan 1st, 10000
const YEAR_10000: u64 = 253402300800;


/// Format a date to be used in a HTTP header field into the provided buffer.
///
/// Dates are formatted as IMF-fixdate: `Fri, 15 May 2015 15:34:21 GMT`.
/// This is a fixed-width format, so this function will always overwrite the entire buffer.
///
/// Since this is a fixed-width format, it does not support dates greater
/// than year 9999.
pub fn format(secs_since_epoch: u64, buffer: &mut [u8; 29]) -> Result<(), TooFuturistic> {
    if secs_since_epoch >= YEAR_10000 {
        return Err(TooFuturistic);
    }

    /* 2000-03-01 (mod 400 year, immediately after feb29 */
    const LEAPOCH: i64 = 11017;
    const DAYS_PER_400Y: i64 = 365 * 400 + 97;
    const DAYS_PER_100Y: i64 = 365 * 100 + 24;
    const DAYS_PER_4Y: i64 = 365 * 4 + 1;

    let days = (secs_since_epoch / 86400) as i64 - LEAPOCH;
    let secs_of_day = secs_since_epoch % 86400;

    let sec = (secs_of_day % 60) as u8;
    let min = ((secs_of_day % 3600) / 60) as u8;
    let hour = (secs_of_day / 3600) as u8;

    let mut qc_cycles = days / DAYS_PER_400Y;
    let mut remdays = days % DAYS_PER_400Y;

    if remdays < 0 {
        remdays += DAYS_PER_400Y;
        qc_cycles -= 1;
    }

    let mut c_cycles = remdays / DAYS_PER_100Y;
    if c_cycles == 4 {
        c_cycles -= 1;
    }
    remdays -= c_cycles * DAYS_PER_100Y;

    let mut q_cycles = remdays / DAYS_PER_4Y;
    if q_cycles == 25 {
        q_cycles -= 1;
    }
    remdays -= q_cycles * DAYS_PER_4Y;

    let mut remyears = remdays / 365;
    if remyears == 4 {
        remyears -= 1;
    }
    remdays -= remyears * 365;

    let mut year = 2000 + remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

    let months = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];
    let mut mon = 0;
    for mon_len in months.iter() {
        mon += 1;
        if remdays < *mon_len {
            break;
        }
        remdays -= *mon_len;
    }
    let mday = remdays + 1;
    let mon = if mon + 2 > 12 {
        year += 1;
        mon - 10
    } else {
        mon + 2
    };

    let mut wday = (3 + days) % 7;
    if wday <= 0 {
        wday += 7
    };

    let wday = match wday {
        1 => b"Mon",
        2 => b"Tue",
        3 => b"Wed",
        4 => b"Thu",
        5 => b"Fri",
        6 => b"Sat",
        7 => b"Sun",
        _ => unreachable!(),
    };

    let month = match mon {
        1 => b"Jan",
        2 => b"Feb",
        3 => b"Mar",
        4 => b"Apr",
        5 => b"May",
        6 => b"Jun",
        7 => b"Jul",
        8 => b"Aug",
        9 => b"Sep",
        10 => b"Oct",
        11 => b"Nov",
        12 => b"Dec",
        _ => unreachable!(),
    };

    *buffer = *b"   , 00     0000 00:00:00 GMT";
    buffer[0] = wday[0];
    buffer[1] = wday[1];
    buffer[2] = wday[2];
    buffer[5] = b'0' + (mday / 10) as u8;
    buffer[6] = b'0' + (mday % 10) as u8;
    buffer[8] = month[0];
    buffer[9] = month[1];
    buffer[10] = month[2];
    buffer[12] = b'0' + (year / 1000) as u8;
    buffer[13] = b'0' + (year / 100 % 10) as u8;
    buffer[14] = b'0' + (year / 10 % 10) as u8;
    buffer[15] = b'0' + (year % 10) as u8;
    buffer[17] = b'0' + (hour / 10) as u8;
    buffer[18] = b'0' + (hour % 10) as u8;
    buffer[20] = b'0' + (min / 10) as u8;
    buffer[21] = b'0' + (min % 10) as u8;
    buffer[23] = b'0' + (sec / 10) as u8;
    buffer[24] = b'0' + (sec % 10) as u8;

    Ok(())
}

/// Errors that can be produced by the timestamp_to_date_header function
pub struct TooFuturistic;









/// Note that this function ignores the portion of the formatted text corresponding to the day of the week.
/// e.g. the "Sun" in "Sun, 02 Oct 2016 14:44:11 GMT". This is usually fine as the week day is redundant
/// information, the date also includes the day of the month. A different crate will be needed if you want
/// to fully validate the date format.
pub fn parse(header: &[u8]) -> Result<u64, ParseError> {
    let date = parse_imf_fixdate(header)
        .or_else(|_| parse_rfc850_date(header))
        .or_else(|_| parse_asctime(header))?;

    let is_valid =
        date.sec < 60
        && date.min < 60
        && date.hour < 24
        && date.day > 0
        && date.day < 32
        && date.mon > 0
        && date.mon <= 12
        && date.year >= 1970
        && date.year <= 9999;

    if !is_valid {
        return Err(ParseError);
    }

    let leap_years = ((date.year - 1) - 1968) / 4 - ((date.year - 1) - 1900) / 100 + ((date.year - 1) - 1600) / 400;

    let mut ydays = match date.mon {
        1 => 0,
        2 => 31,
        3 => 59,
        4 => 90,
        5 => 120,
        6 => 151,
        7 => 181,
        8 => 212,
        9 => 243,
        10 => 273,
        11 => 304,
        12 => 334,
        _ => unreachable!(),
    };
    ydays += date.day as u64;
    ydays -= 1;

    let is_leap_year = date.year % 4 == 0 && (date.year % 100 != 0 || date.year % 400 == 0);
    if is_leap_year && date.mon > 2 {
        ydays += 1;
    }

    let days = (date.year as u64 - 1970) * 365 + leap_years as u64 + ydays;

    let timestamp = date.sec as u64 + date.min as u64 * 60 + date.hour as u64 * 3600 + days * 86400;

    Ok(timestamp)
}

/// Error returned from the `parse` function indicating that the input text was not valid.
#[derive(Debug, Eq, PartialEq)]
pub struct ParseError;









#[derive(Debug, Copy, Clone)]
struct HttpDate {
    sec: u8, // 0...59
    min: u8, // 0...59
    hour: u8, // 0...23
    day: u8, // 1...31
    mon: u8, // 1...12
    year: u16, // 1970...9999
}


fn toint_1(x: u8) -> Result<u8, ParseError> {
    let result = x.wrapping_sub(b'0');
    if result < 10 {
        Ok(result)
    } else {
        Err(ParseError)
    }
}

fn toint_2(s: &[u8]) -> Result<u8, ParseError> {
    let high = s[0].wrapping_sub(b'0');
    let low = s[1].wrapping_sub(b'0');

    if high < 10 && low < 10 {
        Ok(high * 10 + low)
    } else {
        Err(ParseError)
    }
}

fn toint_4(s: &[u8]) -> Result<u16, ParseError> {
    let a = u16::from(s[0].wrapping_sub(b'0'));
    let b = u16::from(s[1].wrapping_sub(b'0'));
    let c = u16::from(s[2].wrapping_sub(b'0'));
    let d = u16::from(s[3].wrapping_sub(b'0'));

    if a < 10 && b < 10 && c < 10 && d < 10 {
        Ok(a * 1000 + b * 100 + c * 10 + d)
    } else {
        Err(ParseError)
    }
}

// Example: `Sun, 06 Nov 1994 08:49:37 GMT`
fn parse_imf_fixdate(s: &[u8]) -> Result<HttpDate, ParseError> {
    if s.len() != 29 || &s[25..] != b" GMT" || s[16] != b' ' || s[19] != b':' || s[22] != b':' {
        return Err(ParseError);
    }

    let date = HttpDate {
        sec: toint_2(&s[23..25])?,
        min: toint_2(&s[20..22])?,
        hour: toint_2(&s[17..19])?,
        day: toint_2(&s[5..7])?,
        mon: match &s[7..12] {
            b" Jan " => 1,
            b" Feb " => 2,
            b" Mar " => 3,
            b" Apr " => 4,
            b" May " => 5,
            b" Jun " => 6,
            b" Jul " => 7,
            b" Aug " => 8,
            b" Sep " => 9,
            b" Oct " => 10,
            b" Nov " => 11,
            b" Dec " => 12,
            _ => return Err(ParseError),
        },
        year: toint_4(&s[12..16])?,
    };

    Ok(date)
}

// Example: `Sunday, 06-Nov-94 08:49:37 GMT`
fn parse_rfc850_date(s: &[u8]) -> Result<HttpDate, ParseError> {
    if s.len() < 23 {
        return Err(ParseError);
    }

    fn wday<'a>(s: &'a [u8], name: &'static [u8]) -> Option<&'a [u8]> {
        if &s[0..name.len()] == name {
            return Some(&s[name.len()..]);
        }

        None
    }

    let s = wday(s, b"Monday, ")
        .or_else(|| wday(s, b"Tuesday, "))
        .or_else(|| wday(s, b"Wednesday, "))
        .or_else(|| wday(s, b"Thursday, "))
        .or_else(|| wday(s, b"Friday, "))
        .or_else(|| wday(s, b"Saturday, "))
        .or_else(|| wday(s, b"Sunday, "))
        .ok_or(ParseError)?;

    if s.len() != 22 || s[12] != b':' || s[15] != b':' || &s[18..22] != b" GMT" {
        return Err(ParseError);
    }

    let mut year = u16::from(toint_2(&s[7..9])?);
    if year < 70 {
        year += 2000;
    } else {
        year += 1900;
    }

    let date = HttpDate {
        sec: toint_2(&s[16..18])?,
        min: toint_2(&s[13..15])?,
        hour: toint_2(&s[10..12])?,
        day: toint_2(&s[0..2])?,
        mon: match &s[2..7] {
            b"-Jan-" => 1,
            b"-Feb-" => 2,
            b"-Mar-" => 3,
            b"-Apr-" => 4,
            b"-May-" => 5,
            b"-Jun-" => 6,
            b"-Jul-" => 7,
            b"-Aug-" => 8,
            b"-Sep-" => 9,
            b"-Oct-" => 10,
            b"-Nov-" => 11,
            b"-Dec-" => 12,
            _ => return Err(ParseError),
        },
        year,
    };

    Ok(date)
}

// Example: `Sun Nov  6 08:49:37 1994`
fn parse_asctime(s: &[u8]) -> Result<HttpDate, ParseError> {
    if s.len() != 24 || s[10] != b' ' || s[13] != b':' || s[16] != b':' || s[19] != b' ' {
        return Err(ParseError);
    }

    let date = HttpDate {
        sec: toint_2(&s[17..19])?,
        min: toint_2(&s[14..16])?,
        hour: toint_2(&s[11..13])?,
        day: {
            let x = &s[8..10];
            {
                if x[0] == b' ' {
                    toint_1(x[1])
                } else {
                    toint_2(x)
                }
            }?
        },
        mon: match &s[4..8] {
            b"Jan " => 1,
            b"Feb " => 2,
            b"Mar " => 3,
            b"Apr " => 4,
            b"May " => 5,
            b"Jun " => 6,
            b"Jul " => 7,
            b"Aug " => 8,
            b"Sep " => 9,
            b"Oct " => 10,
            b"Nov " => 11,
            b"Dec " => 12,
            _ => return Err(ParseError),
        },
        year: toint_4(&s[20..24])?,
    };

    Ok(date)
}




#[cfg(test)]
mod test {
    use proptest::prelude::*;
    use crate::*;

    #[test]
    fn test_parse_static() {
        let success = [
            // Same day, different formats to parse
            (784111777, "Sunday, 06-Nov-94 08:49:37 GMT"),
            (784111777, "Sun Nov  6 08:49:37 1994"),
            (784111777, "Sun, 06 Nov 1994 08:49:37 GMT"),

            // Random additional day to test
            (1475419451, "Sun, 02 Oct 2016 14:44:11 GMT"),

            // Yes, the world ends on a Friday. I checked. Kinda funny really. I would have expected a Monday.
            (253402300799, "Fri, 31 Dec 9999 23:59:59 GMT"),

            (0, "Thu, 01 Jan 1970 00:00:00 GMT"), // The epoch
            (1, "Thu, 01 Jan 1970 00:00:01 GMT"), // The second after the epoch

            (68169599, "Mon, 28 Feb 1972 23:59:59 GMT"), // The second before the first leap year after the epoch
            (68169600, "Tue, 29 Feb 1972 00:00:00 GMT"), // The second of the first leap year
            (68169601, "Tue, 29 Feb 1972 00:00:01 GMT"), // The second after the first leap year
            (68255999, "Tue, 29 Feb 1972 23:59:59 GMT"), // The last second of the first leap year
            (68256000, "Wed, 01 Mar 1972 00:00:00 GMT"), // The first second of the day after the first leap year
            (68256001, "Wed, 01 Mar 1972 00:00:01 GMT"), // The second second of the day after the first leap year

            // Ditto above, but for the second leap year after the epoch
            (194399999, "Sat, 28 Feb 1976 23:59:59 GMT"),
            (194400000, "Sun, 29 Feb 1976 00:00:00 GMT"),
            (194400001, "Sun, 29 Feb 1976 00:00:01 GMT"),
            (194486399, "Sun, 29 Feb 1976 23:59:59 GMT"),
            (194486400, "Mon, 01 Mar 1976 00:00:00 GMT"),
            (194486401, "Mon, 01 Mar 1976 00:00:01 GMT"),

            // 2000 would be a leap year, but since it's a multiple of 100 it's not.
            // Except that it's a multiple of 400 too, so it is.
            // Sounds like a good thing to test.
            (951782399, "Mon, 28 Feb 2000 23:59:59 GMT"), // Second before the not-leap-year-but-actually-leap-year
            (951782400, "Tue, 29 Feb 2000 00:00:00 GMT"), // Second of the not-leap-year-but-actually-leap-year
            (951782401, "Tue, 29 Feb 2000 00:00:01 GMT"), // Second after the not-leap-year-but-actually-leap-year, just for good measure. You can't hide from me evil bugs!
        ];

        let mut buffer = [0u8; 29];
        for (index, (timestamp, formatted)) in success.into_iter().enumerate() {
            assert_eq!(parse(formatted.as_bytes()), Ok(timestamp), "{formatted} parses as {timestamp}");

            // Format always fromats to IMF, but the first two test cases are a different format
            if index >= 2 {
                assert!(format(timestamp, &mut buffer).is_ok(), "{timestamp} formats successfully");
                assert_eq!(&buffer, formatted.as_bytes(), "{timestamp} formats as {formatted}");
            }
        }


        let fail = [
            "Sat, 01 Jan 10000 00:00:00", // First second that can't be represented in true IMF format
            "Wed, 31 Dec 1969 00:00:00 GMT", // day before the epoch
            "Wed, 31 Dec 1969 23:59:59 GMT", // one second before the epoch
            "Sun Nov 10 08:00:00 1000", // Far too long before the epoch. Time probably didn't exist back then.
            "Sun Nov 10 08*00:00 2000", // Invalid character
            "Sunday, 06-Nov-94 08+49:37 GMT", // Invalid character
            ".Sun, 06 Nov 1994 08:49:37 GMT", // Leading invalid character
            "Sun, 06 Nov 1994 08:49:37 GMT.", // Trailing invalid character
        ];

        for formatted in fail {
            assert_eq!(parse(formatted.as_bytes()), Err(ParseError), "{formatted} fails to parse");
        }


        let rolling = [
            (0, "Thu, 01 Jan 1970 00:00:00 GMT"),
            (3600, "Thu, 01 Jan 1970 01:00:00 GMT"), // Add one hour
            (86400, "Fri, 02 Jan 1970 01:00:00 GMT"), // Add one day
            (2592000, "Sun, 01 Feb 1970 01:00:00 GMT"), // Add 30 days
            (2592000, "Tue, 03 Mar 1970 01:00:00 GMT"), // Add 30 days again (this tests February has 28 days in 1970, which was not a leap year)
            (31536005, "Wed, 03 Mar 1971 01:00:05 GMT"), // Add 365 days + 5 seconds
            (15552000, "Mon, 30 Aug 1971 01:00:05 GMT"), // Add 180 days
            (6048000, "Mon, 08 Nov 1971 01:00:05 GMT"), // Add 70 days
            (864000000, "Fri, 26 Mar 1999 01:00:05 GMT"), // Add 10,000 days
        ];

        let mut timestamp = 0;
        for (add_amount, formatted) in rolling {
            timestamp += add_amount;
            assert_eq!(parse(formatted.as_bytes()), Ok(timestamp), "offset {add_amount} formats as {formatted}");
        }
    }


    proptest! {
        #[test]
        fn test_imf_parse(
            week_day in "(Sun|Mon|Tue|Wed|Thu|Fri|Sat)",
            day in 1..=31,
            month in "(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)",
            year in 1970..=9999,
            hour in 0..=23,
            minute in 0..=59,
            second in 0..=59,
        ) {
            let text = format!("{}, {:0>2} {} {} {:0>2}:{:0>2}:{:0>2} GMT", week_day, day, month, year, hour, minute, second);
            let result = parse(text.as_bytes());
            assert!(result.is_ok());
            assert!(result.unwrap() < YEAR_10000);
        }

        #[test]
        fn test_rfc850_parse(
            week_day in "(Sunday|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday)",
            day in 1..=31,
            month in "(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)",
            year in 70..=99,
            hour in 0..=23,
            minute in 0..=59,
            second in 0..=59,
        ) {
            let text = format!("{}, {:0>2}-{}-{:0>2} {:0>2}:{:0>2}:{:0>2} GMT", week_day, day, month, year, hour, minute, second);
            let result = parse(text.as_bytes());
            assert!(result.is_ok());
            assert!(result.unwrap() < YEAR_10000);
        }

        #[test]
        // Example: `Sun Nov  6 08:49:37 1994`
        fn test_asc_parse(
            week_day in "(Sun|Mon|Tue|Wed|Thu|Fri|Sat)",
            month in "(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)",
            day in 1..=31,
            year in 1970..=9999,
            hour in 0..=23,
            minute in 0..=59,
            second in 0..=59,
        ) {
            let text = format!("{} {} {: >2} {:0>2}:{:0>2}:{:0>2} {}", week_day, month, day, hour, minute, second, year);
            let result = parse(text.as_bytes());
            assert!(result.is_ok());
            assert!(result.unwrap() < YEAR_10000);
        }

        #[test]
        fn test_format_props(timestamp in 0..YEAR_10000) {
            let regex = regex::Regex::new(r"(Sun|Mon|Tue|Wed|Thu|Fri|Sat), [0-3]\d (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) (19[7-9]\d|[2-9]\d{3}) ([0-2]\d):([0-5]\d):([0-5]\d) GMT")
                .unwrap();
            let mut buffer = [0; 29];
            let result = format(timestamp, &mut buffer);
            let str_buffer = std::str::from_utf8(&buffer).unwrap();
            assert!(result.is_ok());
            assert!(regex.is_match(str_buffer), "{}", str_buffer);

            let parsed_timestamp = parse(&buffer).unwrap();
            assert_eq!(timestamp, parsed_timestamp);
        }

        #[test]
        fn test_invalid_bits(bits in prop::array::uniform29(0u8..)) {
            // This test assumes that the chances of actually generating a random
            // but valid bit pattern across 29 bytes is effectively impossible.
            assert!(parse(&bits).is_err());
        }
    }
}