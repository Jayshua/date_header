//! Date and time utils for HTTP.
//!
//! Multiple HTTP header fields store timestamps.
//! For example a response created on May 15, 2015 may contain the header
//! `Date: Fri, 15 May 2015 15:34:21 GMT`. Since the timestamp does not
//! contain any timezone or leap second information it is equvivalent to
//! writing 1431696861 Unix time. Rust’s `SystemTime` is used to store
//! these timestamps.
//!
//! This crate provides two public functions:
//!
//! * `parse_http_date` to parse a HTTP datetime string to a system time
//! * `fmt_http_date` to format a system time to a IMF-fixdate
//!
//! In addition it exposes the `HttpDate` type that can be used to parse
//! and format timestamps. Convert a sytem time to `HttpDate` and vice versa.
//! The `HttpDate` (8 bytes) is smaller than `SystemTime` (16 bytes) and
//! using the display impl avoids a temporary allocation.
#![forbid(unsafe_code)]
#![no_std]


// No alloc, no std, no panic, simplified version of httpdate



/// Format a date to be used in a HTTP header field into the provided buffer.
///
/// Dates are formatted as IMF-fixdate: `Fri, 15 May 2015 15:34:21 GMT`.
/// This is a fixed-width format, so this function will always overwrite the entire buffer.
///
/// Since this is a fixed-width format, it does not support dates greater
/// than year 9999.
pub fn format(secs_since_epoch: u64, buffer: &mut [u8; 29]) -> Result<(), FormatError> {
    if secs_since_epoch >= 253402300800 {
        return Err(FormatError::YearGreaterThan9999);
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
pub enum FormatError {
    /// The format used for the http Date header does not support years larger than 9999
    YearGreaterThan9999,
}

#[cfg(test)]
#[test]
fn test_format() {
    let mut buffer = [0u8; 29];

    let tests = [
        (0, "Thu, 01 Jan 1970 00:00:00 GMT"),
        (1475419451, "Sun, 02 Oct 2016 14:44:11 GMT"),
    ];

    for (timestamp, formatted) in tests {
        assert!(format(timestamp, &mut buffer).is_ok(), "{timestamp} formats successfully");
        assert_eq!(&buffer, formatted.as_bytes(), "{timestamp} formats as {formatted}");
    }
}








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
#[test]
fn test_parse() {
    let success = [
        (784111777, "Sun, 06 Nov 1994 08:49:37 GMT"),
        (784111777, "Sunday, 06-Nov-94 08:49:37 GMT"),
        (784111777, "Sun Nov  6 08:49:37 1994"),
        (1475419451, "Sun, 02 Oct 2016 14:44:11 GMT"),
    ];
    for (timestamp, formatted) in success {
        assert_eq!(parse(formatted.as_bytes()), Ok(timestamp), "{timestamp} is the parse of {formatted}");
    }

    let fail = [
        "Sun Nov 10 08:00:00 1000",
        "Sun Nov 10 08*00:00 2000",
        "Sunday, 06-Nov-94 08+49:37 GMT",
    ];
    for formatted in fail {
        assert_eq!(parse(formatted.as_bytes()), Err(ParseError), "{formatted} fails to parse");
    }

    let rolling = [
        (0, "Thu, 01 Jan 1970 00:00:00 GMT"),
        (3600, "Thu, 01 Jan 1970 01:00:00 GMT"),
        (86400, "Fri, 02 Jan 1970 01:00:00 GMT"),
        (2592000, "Sun, 01 Feb 1970 01:00:00 GMT"),
        (2592000, "Tue, 03 Mar 1970 01:00:00 GMT"),
        (31536005, "Wed, 03 Mar 1971 01:00:05 GMT"),
        (15552000, "Mon, 30 Aug 1971 01:00:05 GMT"),
        (6048000, "Mon, 08 Nov 1971 01:00:05 GMT"),
        (864000000, "Fri, 26 Mar 1999 01:00:05 GMT"),
    ];
    let mut timestamp = 0;
    for (add_amount, formatted) in rolling {
        timestamp += add_amount;
        assert_eq!(parse(formatted.as_bytes()), Ok(timestamp), "offset {add_amount} formats as {formatted}");
    }
}
