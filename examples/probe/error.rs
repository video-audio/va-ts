use std::borrow::Cow;
use std::convert::Into;
use std::error::Error as StdError;
use std::fmt;
use std::fmt::Error as FmtError;
use std::io::Error as IoError;
use std::result::Result as StdResult;
use std::str::Utf8Error;

use ts::error::Error as TsError;

pub type Result<T> = StdResult<T, Error>;

macro_rules! from {
    ($src:path, $dst:path) => {
        impl From<$src> for Error {
            fn from(err: $src) -> Error {
                Error::new($dst(err))
            }
        }
    };
}

#[derive(Debug)]
pub enum Kind {
    InputUrlMissingHost,
    InputUrlHostMustBeDomain,
    Io(IoError),
    Fmt(FmtError),
    Encoding(Utf8Error),
    SyncPoison, // TODO: rewrite
    Ts(TsError),

    Unknown(Box<dyn StdError + Send + Sync>),
}

pub struct Error {
    pub kind: Kind,
    pub details: Option<Cow<'static, str>>,
}

impl Error {
    pub fn new(kind: Kind) -> Error {
        Error {
            kind: kind,
            details: None,
        }
    }

    pub fn new_with_details<I>(kind: Kind, details: I) -> Error
    where
        I: Into<Cow<'static, str>>,
    {
        Error {
            kind: kind,
            details: Some(details.into()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"(:error {:?} ("{}""#, self.kind, self.description())?;

        if let Some(details) = self.details.as_ref() {
            write!(f, r#" "{}""#, details)?;
        }

        write!(f, "))")
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self.kind {
            Kind::InputUrlMissingHost => "missing host inside input URL",
            Kind::InputUrlHostMustBeDomain => "provided host must be valid domain",
            Kind::Encoding(ref err) => err.description(),
            Kind::Io(ref err) => err.description(),
            Kind::Fmt(ref err) => err.description(),
            Kind::SyncPoison => "sync lock/condvar poison error",
            Kind::Ts(ref err) => err.description(),

            Kind::Unknown(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match self.kind {
            Kind::InputUrlMissingHost => None,
            Kind::InputUrlHostMustBeDomain => None,
            Kind::Encoding(ref err) => Some(err),
            Kind::Io(ref err) => Some(err),
            Kind::Fmt(ref err) => Some(err),
            Kind::SyncPoison => None,
            Kind::Ts(ref err) => Some(err),

            Kind::Unknown(ref err) => Some(err.as_ref()),
        }
    }
}

from!(Utf8Error, Kind::Encoding);
from!(IoError, Kind::Io);
from!(FmtError, Kind::Fmt);
from!(TsError, Kind::Ts);

impl<B> From<Box<B>> for Error
where
    B: StdError + Send + Sync + 'static,
{
    fn from(err: Box<B>) -> Error {
        Error::new(Kind::Unknown(err))
    }
}
