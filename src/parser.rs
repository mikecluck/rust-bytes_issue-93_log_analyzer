use crate::logs::{self, LogEntry};

struct LogEntriesParser<'a> {
    data: &'a [u8],
}

impl<'a> LogEntriesParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Iterator for LogEntriesParser<'a> {
    type Item = logs::LogEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }

        let (unparsed_data, entry) = parse_log_entry(self.data).unwrap();
        self.data = unparsed_data;
        return Some(entry);
    }
}

fn parse_log_entry(input: &[u8]) -> nom::IResult<&[u8], LogEntry> {
    fn parse_line(input: &[u8]) -> nom::IResult<&[u8], &str> {
        // Keep going until null or newline
        let (input, parse_result) =
            nom::bytes::complete::take_till(|b| b == 0x00 || b == 0x0A)(input)?;
        let (input, _) = nom::bytes::complete::take_while(|b| b == 0x00 || b == 0x0A)(input)?;
        Ok((input, str::from_utf8(parse_result).unwrap()))
    }

    let (input, line) = parse_line(input)?;
    let (line, timestamp) = parse_timestamp(line).map_err(|err| {
        err.map(|e| nom::error::Error {
            input,
            code: e.code,
        })
    })?;
    let (line, hostname) = parse_hostname(line).map_err(|err| {
        err.map(|e| nom::error::Error {
            input,
            code: e.code,
        })
    })?;
    let (line, process_name) = parse_process_name(line).map_err(|err| {
        err.map(|e| nom::error::Error {
            input,
            code: e.code,
        })
    })?;
    let (line, pid) = parse_pid(line).map_err(|err| {
        err.map(|e| nom::error::Error {
            input,
            code: e.code,
        })
    })?;
    Ok((
        input,
        LogEntry {
            timestamp,
            hostname,
            process_name,
            pid,
            message: line.to_string(),
        },
    ))
}

fn parse_timestamp(input: &str) -> nom::IResult<&str, String> {
    fn parse_month(input: &str) -> nom::IResult<&str, &str> {
        nom::bytes::complete::take_while(|c: char| c.is_alphabetic())(input)
    }

    fn parse_day(input: &str) -> nom::IResult<&str, &str> {
        nom::bytes::complete::take_while(|c: char| c.is_digit(10))(input)
    }

    fn parse_time(input: &str) -> nom::IResult<&str, &str> {
        nom::bytes::complete::take_while(|c: char| c.is_digit(10) || c == ':')(input)
    }

    let (input, month) = parse_month(input)?;
    let (input, _) = parse_whitespace(input)?;
    let (input, day) = parse_day(input)?;
    let (input, _) = parse_whitespace(input)?;
    let (input, time) = parse_time(input)?;
    let (input, _) = parse_whitespace(input)?;
    Ok((input, format!("{} {} {}", month, day, time)))
}

fn parse_hostname(input: &str) -> nom::IResult<&str, String> {
    let (input, result) = nom::bytes::complete::take_while(|c: char| !c.is_whitespace())(input)?;
    let (input, _) = parse_whitespace(input)?;
    Ok((input, result.to_string()))
}

fn parse_process_name(input: &str) -> nom::IResult<&str, String> {
    let (input, result) = nom::bytes::complete::take_while(|c: char| c != '[')(input)?;
    Ok((input, result.to_string()))
}

fn parse_pid(input: &str) -> nom::IResult<&str, u32> {
    let (input, _) = nom::bytes::complete::take(1usize)(input)?; // [
    let (input, result) = nom::bytes::complete::take_while(|c: char| c.is_digit(10))(input)?;
    let (input, _) = nom::bytes::complete::take(1usize)(input)?; // ]
    let (input, _) = nom::bytes::complete::take(1usize)(input)?; // :
    let (input, _) = parse_whitespace(input)?;
    Ok((input, result.parse().expect("Failed to parse PID")))
}

fn parse_whitespace(input: &str) -> nom::IResult<&str, &str> {
    nom::bytes::complete::take_while(|c: char| c.is_whitespace())(input)
}

pub(crate) fn parse(file: std::fs::File) -> logs::LogStats {
    let data = unsafe { memmap2::Mmap::map(&file).unwrap() };
    let parser = LogEntriesParser::new(&data);
    let mut stats_builder = logs::LogStatsBuilder::new();
    for entry in parser {
        stats_builder.add_log_entry(entry);
    }
    stats_builder.build()
}
