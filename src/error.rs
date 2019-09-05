use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Kind {
    SyncByte(u8),
    Buf(usize, usize),
    PESStartCode(u32),
    SectionSyntaxIndicatorNotSet,
    AnnexA2EmptyBuf,
    AnnexA2UnsupportedEncoding,
    AnnexA2Decode,
    AnnexA2TableA3Unexpected(u8),
    AnnexA2TableA4Buf(usize, usize),
    AnnexA2TableA4Unexpected(u8),
    AnnexCBuf(usize, usize),

    Io(IoError),
}

pub struct Error(Kind);

impl Error {
    pub fn new(kind: Kind) -> Error {
        Error(kind)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#"(:error ({:?}) (:txt "{}""#,
            self.0,
            self.description()
        )?;

        match self.0 {
            Kind::SyncByte(b) => write!(f, " (:got 0x{:02X})", b)?,
            Kind::Buf(actual, expected) => {
                write!(f, " (:sz-actual {} :sz-expected {})", actual, expected)?
            }
            Kind::PESStartCode(actual) => write!(f, " (:actual 0x{:08X})", actual)?,

            Kind::AnnexA2TableA3Unexpected(b) => write!(f, " (:got 0x{:02X})", b)?,
            Kind::AnnexA2TableA4Buf(actual, expected) => {
                write!(f, " (:sz-actual {} :sz-expected {})", actual, expected)?
            }
            Kind::AnnexA2TableA4Unexpected(b) => write!(f, " (:got 0x{:02X})", b)?,

            Kind::AnnexCBuf(actual, expected) => {
                write!(f, " (:sz-actual {} :sz-expected {})", actual, expected)?
            }

            _ => {}
        }

        write!(f, "))")
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self.0 {
            Kind::SyncByte(..) => "expected sync byte as first element",
            Kind::Buf(..) => "buffer is too small, more data required",
            Kind::PESStartCode(..) => "(pes) unexpected start code",
            Kind::SectionSyntaxIndicatorNotSet => "(psi) section-syntax-indicator must be set",

            Kind::AnnexA2UnsupportedEncoding => "(annex-a2) unsupported encoding",
            Kind::AnnexA2Decode => "(annex-a2) decode error",
            Kind::AnnexA2EmptyBuf => "(annex-a2 parse) got empty character buffer",
            Kind::AnnexA2TableA3Unexpected(..) => "(annex-a2 table-a3 parse) unexpected value",
            Kind::AnnexA2TableA4Buf(..) => {
                "(annex-a2 table-a4 parse) buffer is too small, more data required"
            }
            Kind::AnnexA2TableA4Unexpected(..) => "(annex-a2 table-a4 parse) unexpected value",

            Kind::AnnexCBuf(..) => "(annex-c parse) buffer is too small, more data required",

            Kind::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match self.0 {
            Kind::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        match (&self.0, &other.0) {
            (Kind::SyncByte(a1), Kind::SyncByte(a2)) => a1 == a2,
            (Kind::Buf(a1, b1), Kind::Buf(a2, b2)) => a1 == a2 && b1 == b2,
            (Kind::PESStartCode(a1), Kind::PESStartCode(a2)) => a1 == a2,
            (Kind::SectionSyntaxIndicatorNotSet, Kind::SectionSyntaxIndicatorNotSet) => true,
            (Kind::AnnexA2EmptyBuf, Kind::AnnexA2EmptyBuf) => true,
            (Kind::AnnexA2UnsupportedEncoding, Kind::AnnexA2UnsupportedEncoding) => true,
            (Kind::AnnexA2Decode, Kind::AnnexA2Decode) => true,
            (Kind::AnnexA2TableA3Unexpected(a1), Kind::AnnexA2TableA3Unexpected(a2)) => a1 == a2,
            (Kind::AnnexA2TableA4Buf(a1, b1), Kind::AnnexA2TableA4Buf(a2, b2)) => {
                a1 == a2 && b1 == b2
            }
            (Kind::AnnexA2TableA4Unexpected(a1), Kind::AnnexA2TableA4Unexpected(a2)) => a1 == a2,
            (Kind::AnnexCBuf(a1, a2), Kind::AnnexCBuf(b1, b2)) => a1 == a2 && b1 == b2,
            (Kind::Io(..), Kind::Io(..)) => true,
            _ => false,
        }
    }
}
impl Eq for Error {}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::new(Kind::Io(err))
    }
}
