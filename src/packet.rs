use crate::error::{Error, Kind as ErrorKind};
use crate::header::{Adaptation, Header};
use crate::pcr::PCR;
use crate::pid::PID;
use crate::result::Result;

pub struct Packet<'buf> {
    buf: &'buf [u8],
}

impl<'buf> Packet<'buf> {
    pub const SZ: usize = 188;
    const SYNC_BYTE: u8 = 0x47;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> Result<Packet<'buf>> {
        let pkt = Packet { buf };

        pkt.validate()?;

        Ok(pkt)
    }

    #[inline(always)]
    fn validate(&self) -> Result<()> {
        if self.buf.len() != Self::SZ {
            Err(Error::new(ErrorKind::Buf(self.buf.len(), Self::SZ)))
        } else if self.buf[0] != Self::SYNC_BYTE {
            Err(Error::new(ErrorKind::SyncByte(self.buf[0])))
        } else {
            Ok(())
        }
    }

    /// adaptation start position
    #[inline(always)]
    fn buf_pos_adaptation() -> usize {
        Header::SZ
    }

    // TODO: try_seek?
    //       or pos_<name> + seek?
    /// position payload start
    #[inline(always)]
    fn buf_pos_payload(&self, is_section: bool) -> usize {
        let mut pos = Self::buf_pos_adaptation();
        let header = self.header();

        if header.got_adaptation() {
            // TODO: Adaptation::sz(self.buf)
            //       self.adaptation() + self.try_adaptation()
            let adapt = Adaptation::new(self.buf_seek(pos));
            pos += adapt.sz();
        }

        if header.pusi() && is_section {
            // payload data start
            //
            // https://stackoverflow.com/a/27525217
            // From the en300 468 spec:
            //
            // Sections may start at the beginning of the payload of a TS packet,
            // but this is not a requirement, because the start of the first
            // section in the payload of a TS packet is pointed to by the pointer_field.
            //
            // So the section start actually is an offset from the payload:
            //
            // uint8_t* section_start = payload + *payload + 1;
            pos += (self.buf[pos] as usize) + 1;
        }

        pos
    }

    #[inline(always)]
    fn buf_seek(&self, offset: usize) -> &'buf [u8] {
        &self.buf[offset..]
    }

    #[inline(always)]
    fn buf_try_seek(&self, offset: usize) -> Result<&'buf [u8]> {
        if self.buf.len() <= offset {
            Err(Error::new(ErrorKind::Buf(self.buf.len(), Self::SZ)))
        } else {
            Ok(self.buf_seek(offset))
        }
    }

    #[inline(always)]
    fn buf_adaptation(&self) -> Result<&'buf [u8]> {
        self.buf_try_seek(Self::buf_pos_adaptation())
    }

    #[inline(always)]
    fn buf_payload(&self, is_section: bool) -> Result<&'buf [u8]> {
        self.buf_try_seek(self.buf_pos_payload(is_section))
    }

    #[inline(always)]
    pub fn buf_payload_section(&self) -> Result<&'buf [u8]> {
        self.buf_payload(true)
    }

    #[inline(always)]
    pub fn buf_payload_pes(&self) -> Result<&'buf [u8]> {
        self.buf_payload(false)
    }

    // TODO: merge Header and Packet?
    #[inline(always)]
    fn header(&self) -> Header<'buf> {
        Header::new(self.buf)
    }

    #[inline(always)]
    fn adaptation(&self) -> Option<Result<Adaptation<'buf>>> {
        let header = self.header();

        if header.got_adaptation() {
            // TODO: move to macro? or optional-result crate
            match self.buf_adaptation() {
                Ok(buf) => Some(Adaptation::try_new(buf)),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn pid(&self) -> PID {
        self.header().pid()
    }

    #[inline(always)]
    pub fn cc(&self) -> u8 {
        self.header().cc()
    }

    #[inline(always)]
    pub fn pusi(&self) -> bool {
        self.header().pusi()
    }

    #[inline(always)]
    pub fn pcr(&self) -> Result<Option<PCR<'buf>>> {
        self.adaptation()
            .and_then(|res| match res {
                Ok(adapt) => adapt.pcr().map(Ok),
                Err(e) => Some(Err(e)),
            })
            .transpose()
    }

    // TODO: generic pmt, pat method
    #[inline(always)]
    pub fn pat(&self) -> Result<Option<&'buf [u8]>> {
        let header = self.header();

        if !header.got_payload() {
            return Ok(None);
        }

        let res = if self.pid() == PID::PAT {
            // TODO: move to macro? or optional-result crate
            match self.buf_payload_section() {
                Ok(buf) => Some(Ok(buf)),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        };

        res.transpose()
    }

    // TODO: refactoring
    #[inline(always)]
    pub fn pmt(&self, pid: u16) -> Result<Option<&'buf [u8]>> {
        let header = self.header();

        if !header.got_payload() {
            return Ok(None);
        }

        let res = if u16::from(self.pid()) == pid {
            // TODO: move to macro? or optional-result crate
            match self.buf_payload_section() {
                Ok(buf) => Some(Ok(buf)),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        };

        res.transpose()
    }

    // TODO: refactoring
    #[inline(always)]
    pub fn eit(&self) -> Result<Option<&'buf [u8]>> {
        let header = self.header();

        if !header.got_payload() {
            return Ok(None);
        }

        let res = if self.pid() == PID::EIT {
            // TODO: move to macro? or optional-result crate
            match self.buf_payload_section() {
                Ok(buf) => Some(Ok(buf)),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        };

        res.transpose()
    }
}
