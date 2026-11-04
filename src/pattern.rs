use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::str::FromStr;

pub trait Pattern {
	fn fill(&mut self, buffer: &mut [u8]) -> io::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternSpec {
	Zero,
	Ones,
	Random,
	Custom(Vec<u8>),
}

impl FromStr for PatternSpec {
	type Err = String;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value {
			"0" | "zero" => Ok(Self::Zero),
			"1" | "ones" => Ok(Self::Ones),
			"r" | "random" => Ok(Self::Random),
			value if value.starts_with("0x") || value.starts_with("0X") => {
				let hex = &value[2..];
				if hex.is_empty() || hex.len() % 2 != 0 {
					return Err(format!(
						"custom patterns must contain an even number of hexadecimal digits: {value}"
					));
				}
				let mut bytes = Vec::with_capacity(hex.len() / 2);
				let (pairs, remainder) = hex.as_bytes().as_chunks::<2>();
				assert!(remainder.is_empty());
				for pair in pairs {
					let pair =
						std::str::from_utf8(pair).expect("hexadecimal input should be ASCII");
					bytes.push(
						u8::from_str_radix(pair, 16)
							.map_err(|_| format!("invalid hexadecimal pattern: {value}"))?,
					);
				}
				Ok(Self::Custom(bytes))
			}
			_ => Err(format!(
				"invalid pass pattern {value:?}; expected 0, 1, r, or 0x followed by hexadecimal digits"
			)),
		}
	}
}

impl fmt::Display for PatternSpec {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Zero => formatter.write_str("0"),
			Self::Ones => formatter.write_str("1"),
			Self::Random => formatter.write_str("r"),
			Self::Custom(bytes) => {
				formatter.write_str("0x")?;
				for byte in bytes {
					write!(formatter, "{byte:02X}")?;
				}
				Ok(())
			}
		}
	}
}

impl PatternSpec {
	pub fn create(&self) -> io::Result<Box<dyn Pattern>> {
		Ok(match self {
			Self::Zero => Box::new(ZeroPattern),
			Self::Ones => Box::new(OnesPattern),
			Self::Random => Box::new(RandomPattern::new()?),
			Self::Custom(bytes) => Box::new(CustomPattern {
				bytes: bytes.clone(),
				offset: 0,
			}),
		})
	}
}

pub struct RandomPattern {
	source: File,
}

impl RandomPattern {
	pub fn new() -> io::Result<Self> {
		Ok(Self {
			source: File::open("/dev/urandom")?,
		})
	}
}

impl Pattern for RandomPattern {
	fn fill(&mut self, buffer: &mut [u8]) -> io::Result<()> {
		self.source.read_exact(buffer)
	}
}

pub struct ZeroPattern;

impl Pattern for ZeroPattern {
	fn fill(&mut self, buffer: &mut [u8]) -> io::Result<()> {
		buffer.fill(0);
		Ok(())
	}
}

pub struct OnesPattern;

impl Pattern for OnesPattern {
	fn fill(&mut self, buffer: &mut [u8]) -> io::Result<()> {
		buffer.fill(u8::MAX);
		Ok(())
	}
}

struct CustomPattern {
	bytes: Vec<u8>,
	offset: usize,
}

impl Pattern for CustomPattern {
	fn fill(&mut self, buffer: &mut [u8]) -> io::Result<()> {
		for byte in buffer {
			*byte = self.bytes[self.offset];
			self.offset = (self.offset + 1) % self.bytes.len();
		}
		Ok(())
	}
}
