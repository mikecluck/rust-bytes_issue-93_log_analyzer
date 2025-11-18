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

fn ws(input: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    nom::bytes::complete::take_while(|b| (b as char).is_whitespace())(input)
}

fn parse_log_entry(input: &[u8]) -> nom::IResult<&[u8], LogEntry> {
    // timestamp
    let (input, month) = nom::bytes::complete::take_while(|b| (b as char).is_alphabetic())(input)?;
    let (input, _) = ws(input)?;
    let (input, day) = nom::bytes::complete::take_while(|b| (b as char).is_digit(10))(input)?;
    let (input, _) = ws(input)?;
    let (input, time) =
        nom::bytes::complete::take_while(|b| (b as char).is_digit(10) || (b as char) == ':')(
            input,
        )?;
    let (input, _) = ws(input)?;

    // hostname
    let (input, hostname) =
        nom::bytes::complete::take_while(|b| !(b as char).is_whitespace())(input)?;
    let (input, _) = ws(input)?;

    // process name
    let (input, process_name) = nom::bytes::complete::take_while(|b| (b as char) != '[')(input)?;

    // pid
    let (input, _) = nom::bytes::complete::take(1usize)(input)?; // [
    let (input, pid) = nom::bytes::complete::take_while(|b| (b as char).is_digit(10))(input)?;
    let (input, _) = nom::bytes::complete::take(1usize)(input)?; // ]
    let (input, _) = nom::bytes::complete::take(1usize)(input)?; // :
    let (input, _) = ws(input)?;

    // the message
    let (input, message) = nom::bytes::complete::take_till(|b| b == 0x00 || b == 0x0A)(input)?;

    // Eat the rest of the line or file
    let (input, _) = nom::bytes::complete::take_while(|b| b == 0x00 || b == 0x0A)(input)?;

    // convert everything to `str`s
    let month = str::from_utf8(month).unwrap();
    let day = str::from_utf8(day).unwrap();
    let time = str::from_utf8(time).unwrap();
    let hostname = str::from_utf8(hostname).unwrap();
    let process_name = str::from_utf8(process_name).unwrap();
    let pid = str::from_utf8(pid).unwrap();
    let message = str::from_utf8(message).unwrap();

    Ok((
        input,
        LogEntry {
            timestamp: format!("{} {} {}", month, day, time),
            hostname: hostname.to_string(),
            process_name: process_name.to_string(),
            pid: pid.parse().unwrap(),
            message: message.to_string(),
        },
    ))
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
